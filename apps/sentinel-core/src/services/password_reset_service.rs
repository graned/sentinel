//! Password reset service — time-limited, single-use token management.
//!
//! # Token format
//!
//! `pr_<64 hex chars>` — the `pr_` prefix stands for **p**assword **r**eset.
//! The suffix is 32 bytes from `OsRng` encoded as lowercase hex.
//! Only the SHA-256 hash is stored in `password_reset_tokens.token_hash`.
//! The raw token is embedded in the reset link emailed to the user.
//!
//! # Token lifetime
//!
//! Tokens expire **1 hour** after creation.  After expiry, `consume_token` returns
//! `AuthenticationError` — identical to the "already used" and "not found" paths to
//! prevent information leakage about token state.
//!
//! # Session revocation
//!
//! This service only manages token lifecycle.  The calling application layer
//! (`AuthApplication::reset_password`) is responsible for revoking all existing
//! sessions after a successful password reset.

use crate::{DbConnection, PasswordResetToken, PasswordResetTokenRepository, ServiceError};
use chrono::{Duration, Utc};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

/// Manages password-reset token issuance and consumption.
pub struct PasswordResetService {
    repo: Arc<PasswordResetTokenRepository>,
    token_ttl: Duration,
}

impl PasswordResetService {
    pub fn new(repo: Arc<PasswordResetTokenRepository>) -> Self {
        Self {
            repo,
            token_ttl: Duration::hours(1),
        }
    }

    /// Generates a reset token, stores its SHA-256 hash, and returns the raw token for emailing.
    /// Token format: `pr_<64 hex chars>` (32 bytes from OsRng).
    pub async fn create_reset_token(
        &self,
        conn: &mut DbConnection<'_>,
        identity_id: Uuid,
        user_id: Uuid,
    ) -> Result<String, ServiceError> {
        let raw_token = generate_token();
        let token_hash = sha256_hex(&raw_token);

        let now = Utc::now();
        let record = PasswordResetToken {
            reset_token_id: Uuid::new_v4(),
            identity_id,
            token_hash,
            expires_at: now + self.token_ttl,
            used_at: None,
            created_at: now,
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

    /// Validates a raw reset token, marks it as used, and returns the identity_id.
    /// All failure paths return `AuthenticationError` to avoid leaking information.
    pub async fn consume_token<'a>(
        &self,
        conn: &mut DbConnection<'a>,
        raw_token: &'a str,
    ) -> Result<Uuid, ServiceError> {
        use crate::schema::password_reset_tokens::token_hash as col_token_hash;
        use diesel::ExpressionMethods;

        let hash = sha256_hex(raw_token);
        let rows = self
            .repo
            .find_where(conn, col_token_hash.eq(hash))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let record = rows.into_iter().next().ok_or_else(|| {
            ServiceError::AuthenticationError("Invalid password reset token".to_string())
        })?;

        if record.used_at.is_some() {
            return Err(ServiceError::AuthenticationError(
                "Password reset token has already been used".to_string(),
            ));
        }

        if record.expires_at < Utc::now() {
            return Err(ServiceError::AuthenticationError(
                "Password reset token has expired".to_string(),
            ));
        }

        // Mark as used
        #[derive(diesel::AsChangeset)]
        #[diesel(table_name = crate::schema::password_reset_tokens)]
        struct UsedChangeset {
            used_at: Option<chrono::DateTime<Utc>>,
            updated_at: Option<chrono::DateTime<Utc>>,
        }

        self.repo
            .update(
                conn,
                record.reset_token_id,
                UsedChangeset {
                    used_at: Some(Utc::now()),
                    updated_at: Some(Utc::now()),
                },
            )
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(record.identity_id)
    }
}

/// Generate a `pr_<hex>` password-reset token from 32 bytes of OS randomness.
fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    format!("pr_{}", hex::encode(bytes))
}

/// Compute the lowercase hex SHA-256 digest of `s`.
fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}
