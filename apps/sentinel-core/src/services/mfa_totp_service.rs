//! TOTP-based multi-factor authentication service.
//!
//! # Enrollment flow
//!
//! 1. **`start_enrollment`** — generates a 20-byte random TOTP secret, encrypts it with
//!    XChaCha20-Poly1305, stores the ciphertext in `user_mfa_totp`, and returns an
//!    `otpauth://` URI that the user scans with an authenticator app.
//! 2. **`confirm_enrollment`** — verifies the first TOTP code from the app,
//!    regenerates 8 one-time recovery codes (only their SHA-256 hashes are stored),
//!    and marks MFA as `enabled = true`.
//!
//! # Verification flow (login)
//!
//! `verify` tries the submitted code against the live TOTP window first.
//! If that fails, it falls back to checking recovery codes (and marks the matching
//! code as `used_at = now()`). Only if both paths fail is `MfaInvalidCode` returned.
//!
//! # Secret storage
//!
//! Secrets are encrypted at rest using XChaCha20-Poly1305 (same key as SMTP configs —
//! `CONFIG_ENCRYPTION_KEY`). The ciphertext format is `[24-byte nonce || ciphertext]`.

use crate::{
    DbConnection, ServiceError, UserMfaTotp, UserMfaTotpRepository, UserRecoveryCode,
    UserRecoveryCodeRepository,
};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use chrono::Utc;
use diesel::ExpressionMethods;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use totp_rs::{Algorithm, TOTP};
use uuid::Uuid;

/// XChaCha20 nonce size in bytes (24 bytes = 192 bits).
const NONCE_SIZE: usize = 24;
/// Number of one-time recovery codes generated on MFA enrollment confirmation.
const RECOVERY_CODE_COUNT: usize = 8;

/// Handles TOTP secret management, enrollment, verification, and recovery codes.
pub struct MfaTotpService {
    totp_repository: Arc<UserMfaTotpRepository>,
    recovery_code_repository: Arc<UserRecoveryCodeRepository>,
    /// 32-byte key used for XChaCha20-Poly1305 encryption of TOTP secrets at rest.
    encryption_key: [u8; 32],
}

/// Output of [`MfaTotpService::start_enrollment`].
pub struct StartMfaEnrollmentData {
    /// `otpauth://totp/...` URI — encode as a QR code for the user's authenticator app.
    pub otpauth_uri: String,
}

/// Output of [`MfaTotpService::confirm_enrollment`].
pub struct ConfirmMfaEnrollmentData {
    /// Plain-text one-time recovery codes, shown to the user exactly once.
    /// Only their SHA-256 hashes are stored in the database.
    pub recovery_codes: Vec<String>,
}

impl MfaTotpService {
    pub fn new(
        totp_repository: Arc<UserMfaTotpRepository>,
        recovery_code_repository: Arc<UserRecoveryCodeRepository>,
        encryption_key: [u8; 32],
    ) -> Self {
        Self {
            totp_repository,
            recovery_code_repository,
            encryption_key,
        }
    }

    pub async fn start_enrollment(
        &self,
        conn: &mut DbConnection<'_>,
        user_id: Uuid,
        email: &str,
    ) -> Result<StartMfaEnrollmentData, ServiceError> {
        // Generate secret and build otpauth URI in a sync block (no .await inside)
        let (encrypted, otpauth_uri) = {
            let mut secret_bytes = vec![0u8; 20];
            OsRng.fill_bytes(&mut secret_bytes);

            let totp = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret_bytes.clone(),
                Some("Sentinel".to_string()),
                email.to_string(),
            )
            .map_err(|e| ServiceError::InternalError(format!("TOTP build error: {}", e)))?;

            let uri = totp.get_url();
            let enc = self.encrypt_bytes(&secret_bytes)?;
            (enc, uri)
        };

        use crate::schema::user_mfa_totp::user_id as uid_col;

        let existing = self
            .totp_repository
            .find_where(conn, uid_col.eq(user_id))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if let Some(existing_row) = existing.into_iter().next() {
            #[derive(diesel::AsChangeset)]
            #[diesel(table_name = crate::schema::user_mfa_totp)]
            struct ResetChangeset {
                secret_encrypted: Vec<u8>,
                enabled: bool,
                enrolled_at: Option<chrono::DateTime<Utc>>,
            }
            let changeset = ResetChangeset {
                secret_encrypted: encrypted,
                enabled: false,
                enrolled_at: None,
            };
            self.totp_repository
                .update(conn, existing_row.user_mfa_totp_id, changeset)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        } else {
            let new_row = UserMfaTotp {
                user_mfa_totp_id: Uuid::new_v4(),
                user_id,
                secret_encrypted: encrypted,
                enabled: false,
                enrolled_at: None,
                last_used_at: None,
                created_at: Utc::now(),
            };
            self.totp_repository
                .create(conn, &new_row)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        Ok(StartMfaEnrollmentData { otpauth_uri })
    }

    pub async fn confirm_enrollment(
        &self,
        conn: &mut DbConnection<'_>,
        user_id: Uuid,
        code: &str,
    ) -> Result<ConfirmMfaEnrollmentData, ServiceError> {
        use crate::schema::user_mfa_totp::user_id as uid_col;

        let rows = self
            .totp_repository
            .find_where(conn, uid_col.eq(user_id))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let row = rows
            .into_iter()
            .next()
            .ok_or_else(|| ServiceError::MfaNotEnrolled("MFA enrollment not started".to_string()))?;

        // Verify the TOTP code in a sync block
        let valid = {
            let secret_bytes = self.decrypt_bytes(&row.secret_encrypted)?;
            let totp = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret_bytes,
                Some("Sentinel".to_string()),
                String::new(),
            )
            .map_err(|e| ServiceError::InternalError(format!("TOTP build error: {}", e)))?;
            totp.check_current(code)
                .map_err(|e| ServiceError::InternalError(format!("TOTP time error: {}", e)))?
        };

        if !valid {
            return Err(ServiceError::MfaInvalidCode("Invalid TOTP code".to_string()));
        }

        // Clear previous recovery codes
        self.recovery_code_repository
            .delete_all_for_user(conn, user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Generate new recovery codes (sync block, no .await)
        let (plain_codes, hashed_rows) = {
            let mut plain_codes = Vec::with_capacity(RECOVERY_CODE_COUNT);
            let mut hashed_rows: Vec<UserRecoveryCode> = Vec::with_capacity(RECOVERY_CODE_COUNT);
            let mut bytes = [0u8; 8];
            for _ in 0..RECOVERY_CODE_COUNT {
                OsRng.fill_bytes(&mut bytes);
                let lo = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
                let hi = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
                let plain = format!("{:08x}-{:08x}", lo, hi);
                let hash = sha256_hex(&plain);
                plain_codes.push(plain);
                hashed_rows.push(UserRecoveryCode {
                    user_recovery_code_id: Uuid::new_v4(),
                    user_id,
                    code_hash: hash,
                    used_at: None,
                    created_at: Utc::now(),
                });
            }
            (plain_codes, hashed_rows)
        };

        for rc_row in &hashed_rows {
            self.recovery_code_repository
                .create(conn, rc_row)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        // Enable MFA
        #[derive(diesel::AsChangeset)]
        #[diesel(table_name = crate::schema::user_mfa_totp)]
        struct EnableChangeset {
            enabled: bool,
            enrolled_at: Option<chrono::DateTime<Utc>>,
        }
        let changeset = EnableChangeset {
            enabled: true,
            enrolled_at: Some(Utc::now()),
        };
        self.totp_repository
            .update(conn, row.user_mfa_totp_id, changeset)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(ConfirmMfaEnrollmentData {
            recovery_codes: plain_codes,
        })
    }

    pub async fn verify(
        &self,
        conn: &mut DbConnection<'_>,
        user_id: Uuid,
        code: &str,
    ) -> Result<(), ServiceError> {
        use crate::schema::user_mfa_totp::user_id as uid_col;

        let rows = self
            .totp_repository
            .find_where(conn, uid_col.eq(user_id))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let row = rows.into_iter().next().ok_or_else(|| {
            ServiceError::MfaNotEnrolled("MFA not enrolled for this user".to_string())
        })?;

        if !row.enabled {
            return Err(ServiceError::MfaNotEnrolled(
                "MFA not enabled for this user".to_string(),
            ));
        }

        // Try TOTP first (sync block)
        let totp_valid = {
            let secret_bytes = self.decrypt_bytes(&row.secret_encrypted)?;
            let totp = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret_bytes,
                Some("Sentinel".to_string()),
                String::new(),
            )
            .map_err(|e| ServiceError::InternalError(format!("TOTP build error: {}", e)))?;
            totp.check_current(code)
                .map_err(|e| ServiceError::InternalError(format!("TOTP time error: {}", e)))?
        };

        if totp_valid {
            #[derive(diesel::AsChangeset)]
            #[diesel(table_name = crate::schema::user_mfa_totp)]
            struct LastUsedChangeset {
                last_used_at: Option<chrono::DateTime<Utc>>,
            }
            let changeset = LastUsedChangeset {
                last_used_at: Some(Utc::now()),
            };
            self.totp_repository
                .update(conn, row.user_mfa_totp_id, changeset)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
            return Ok(());
        }

        // Fall back to recovery code
        let code_hash = sha256_hex(code);
        let found_rc = self
            .find_unused_recovery_code(conn, user_id, code_hash)
            .await?;

        if let Some(rc_row) = found_rc {
            #[derive(diesel::AsChangeset)]
            #[diesel(table_name = crate::schema::user_recovery_codes)]
            struct MarkUsedChangeset {
                used_at: Option<chrono::DateTime<Utc>>,
            }
            let rc_changeset = MarkUsedChangeset {
                used_at: Some(Utc::now()),
            };
            self.recovery_code_repository
                .update(conn, rc_row.user_recovery_code_id, rc_changeset)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
            return Ok(());
        }

        Err(ServiceError::MfaInvalidCode(
            "Invalid MFA code or recovery code".to_string(),
        ))
    }

    pub async fn is_mfa_enabled(
        &self,
        conn: &mut DbConnection<'_>,
        user_id: Uuid,
    ) -> Result<bool, ServiceError> {
        use crate::schema::user_mfa_totp::user_id as uid_col;

        let rows = self
            .totp_repository
            .find_where(conn, uid_col.eq(user_id))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().next().map(|r| r.enabled).unwrap_or(false))
    }

    /// Helper: find an unused recovery code by hash for a given user.
    /// Uses a direct Diesel query to avoid lifetime issues with compound expressions.
    async fn find_unused_recovery_code(
        &self,
        conn: &mut crate::DbConnection<'_>,
        target_user_id: Uuid,
        hash: String,
    ) -> Result<Option<UserRecoveryCode>, ServiceError> {
        use crate::schema::user_recovery_codes::dsl::{
            code_hash as hash_col, used_at as used_at_col, user_id as rc_uid_col,
            user_recovery_codes,
        };
        use diesel::prelude::*;
        use diesel::OptionalExtension;
        use diesel_async::RunQueryDsl;

        user_recovery_codes
            .filter(rc_uid_col.eq(target_user_id))
            .filter(hash_col.eq(hash))
            .filter(used_at_col.is_null())
            .first::<UserRecoveryCode>(conn)
            .await
            .optional()
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Encrypt arbitrary bytes using XChaCha20-Poly1305.
    ///
    /// Output format: `[24-byte nonce || authenticated ciphertext]`.
    /// A fresh random nonce is generated for every call with `OsRng`.
    fn encrypt_bytes(&self, data: &[u8]) -> Result<Vec<u8>, ServiceError> {
        let cipher = XChaCha20Poly1305::new_from_slice(&self.encryption_key)
            .map_err(|_| ServiceError::InternalError("Invalid encryption key".to_string()))?;

        let nonce: XNonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| ServiceError::InternalError(e.to_string()))?;

        // Prepend the nonce so the decryption side can extract it.
        let nonce_bytes: [u8; NONCE_SIZE] = nonce.into();
        let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);

        Ok(output)
    }

    /// Decrypt bytes previously encrypted by [`encrypt_bytes`].
    ///
    /// Expects the first `NONCE_SIZE` bytes to be the XChaCha20 nonce,
    /// followed by the authenticated ciphertext.
    fn decrypt_bytes(&self, encrypted: &[u8]) -> Result<Vec<u8>, ServiceError> {
        if encrypted.len() < NONCE_SIZE {
            return Err(ServiceError::InternalError(
                "Invalid encrypted data".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
        let nonce = XNonce::from_slice(nonce_bytes);

        let cipher = XChaCha20Poly1305::new_from_slice(&self.encryption_key)
            .map_err(|_| ServiceError::InternalError("Invalid encryption key".to_string()))?;

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ServiceError::InternalError(e.to_string()))
    }
}

/// Compute the SHA-256 digest of a string and return it as a lowercase hex string.
/// Used to hash recovery codes before storing them so plaintext is never persisted.
fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}
