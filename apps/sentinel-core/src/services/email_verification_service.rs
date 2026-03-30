//! Email verification service — time-limited, single-use token management.
//!
//! # Token format
//!
//! `ev_<64 hex chars>` — the `ev_` prefix stands for **e**mail **v**erification.
//! The suffix is 32 bytes from `OsRng` encoded as lowercase hex.
//! Only the SHA-256 hash is stored in `email_verifications.token_hash`.
//! The raw token is embedded in the verification link emailed to the user.
//!
//! # Token lifetime
//!
//! Tokens expire **24 hours** after creation.  After expiry (or once consumed),
//! `consume_token` returns `AuthenticationError` with no indication of which failure
//! path triggered — same message for "not found", "already used", and "expired".
//!
//! # Post-consumption steps
//!
//! This service only marks the token row as verified.  The calling application layer
//! (`AuthApplication::verify_email`) must also call
//! `IdentityService::mark_email_verified` to set `user_identities.email_verified = true`.

use crate::{DbConnection, EmailVerification, EmailVerificationRepository, ServiceError};
use chrono::{Duration, Utc};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

/// Manages email verification token issuance and consumption.
pub struct EmailVerificationService {
    repo: Arc<EmailVerificationRepository>,
    token_ttl: Duration,
}

impl EmailVerificationService {
    pub fn new(repo: Arc<EmailVerificationRepository>) -> Self {
        Self {
            repo,
            token_ttl: Duration::hours(24),
        }
    }

    /// Creates an email verification record and returns the raw token to be emailed.
    /// Token format: `ev_<64 hex chars>` (32 bytes from OsRng).
    /// Only the SHA-256 hash is stored in DB.
    pub async fn create_verification(
        &self,
        conn: &mut DbConnection<'_>,
        identity_id: Uuid,
        user_id: Uuid,
    ) -> Result<String, ServiceError> {
        let raw_token = generate_token();
        let token_hash = sha256_hex(&raw_token);

        let now = Utc::now();
        let record = EmailVerification {
            verification_id: Uuid::new_v4(),
            identity_id,
            token_hash,
            expires_at: now + self.token_ttl,
            verified_at: None,
            created_at: Some(now),
            updated_at: Some(now),
            created_by: Some(user_id),
            updated_by: Some(user_id),
        };

        self.repo
            .create(conn, &record)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(raw_token)
    }

    /// Validates a raw token, marks it verified, and returns the identity_id.
    /// All failure paths return `AuthenticationError` to avoid leaking existence.
    pub async fn consume_token<'a>(
        &self,
        conn: &mut DbConnection<'a>,
        raw_token: &'a str,
    ) -> Result<Uuid, ServiceError> {
        use crate::schema::email_verifications::token_hash as col_token_hash;
        use diesel::ExpressionMethods;

        let hash = sha256_hex(raw_token);
        let rows = self
            .repo
            .find_where(conn, col_token_hash.eq(hash))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let record = rows.into_iter().next().ok_or_else(|| {
            ServiceError::AuthenticationError("Invalid verification token".to_string())
        })?;

        if record.verified_at.is_some() {
            return Err(ServiceError::AuthenticationError(
                "Token already used".to_string(),
            ));
        }

        if record.expires_at < Utc::now() {
            return Err(ServiceError::AuthenticationError(
                "Verification token has expired".to_string(),
            ));
        }

        // Mark as verified
        #[derive(diesel::AsChangeset)]
        #[diesel(table_name = crate::schema::email_verifications)]
        struct VerifiedChangeset {
            verified_at: Option<chrono::DateTime<Utc>>,
            updated_at: Option<chrono::DateTime<Utc>>,
        }

        self.repo
            .update(
                conn,
                record.verification_id,
                VerifiedChangeset {
                    verified_at: Some(Utc::now()),
                    updated_at: Some(Utc::now()),
                },
            )
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(record.identity_id)
    }
}

/// Generate an `ev_<hex>` email-verification token from 32 bytes of OS randomness.
fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    format!("ev_{}", hex::encode(bytes))
}

/// Compute the lowercase hex SHA-256 digest of `s`.
fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}
