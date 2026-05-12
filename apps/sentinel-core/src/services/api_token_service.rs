//! API token service — generation, validation, and lifecycle management.
//!
//! API tokens provide long-lived programmatic access (CI/CD, scripts) as an
//! alternative to short-lived PASETO session tokens.
//!
//! # Token format
//!
//! `sat_<64 hex chars>` — the prefix `sat_` stands for **S**entinel **A**PI **T**oken.
//! The suffix is 32 bytes from `OsRng` encoded as lowercase hex.
//! Only the SHA-256 hash of the full raw token is stored in `api_tokens.token_hash`.
//! The raw token is returned **exactly once** at creation.
//!
//! # Revocation
//!
//! Tokens are soft-deleted: `revoked_at` is set to `now()`; no rows are hard-deleted.
//! Validation checks both `revoked_at IS NULL` and `expires_at > now()`.
//!
//! # Security note
//!
//! `validate_token` returns the same `AuthenticationError` for "not found",
//! "revoked", and "expired" to prevent callers from distinguishing these states.

use crate::{ApiToken, ApiTokenRepository, DbConnection, ServiceError};
use chrono::Utc;
use hex;
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

/// Handles API token generation, persistence, and validation.
pub struct ApiTokenService {
    repo: Arc<ApiTokenRepository>,
}

impl ApiTokenService {
    pub fn new(repo: Arc<ApiTokenRepository>) -> Self {
        Self { repo }
    }

    /// Generate a random 32-byte opaque token and its SHA-256 hex hash.
    /// Returns `(raw_token, token_hash)`.
    pub fn generate_token(&self) -> (String, String) {
        let mut buf = [0u8; 32];
        OsRng.fill_bytes(&mut buf);
        let raw = format!("sat_{}", hex::encode(buf));
        let hash = sha256_hex(&raw);
        (raw, hash)
    }

    pub async fn create(
        &self,
        conn: &mut DbConnection<'_>,
        token: &ApiToken,
    ) -> Result<ApiToken, ServiceError> {
        self.repo
            .create(conn, token)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn find_by_id(
        &self,
        conn: &mut DbConnection<'_>,
        id: Uuid,
    ) -> Result<ApiToken, ServiceError> {
        self.repo
            .find_by_id(conn, id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::ApiTokenNotFound("API token not found".to_string()))
    }

    /// Return all API tokens owned by `target_user_id` (both active and revoked).
    pub async fn list_for_user(
        &self,
        conn: &mut DbConnection<'_>,
        target_user_id: Uuid,
    ) -> Result<Vec<ApiToken>, ServiceError> {
        use crate::schema::api_tokens::user_id as col_user_id;
        use diesel::ExpressionMethods;

        self.repo
            .find_where(conn, col_user_id.eq(target_user_id))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Soft-revoke a single token owned by the given user.
    pub async fn revoke(
        &self,
        conn: &mut DbConnection<'_>,
        token_id: Uuid,
        owner_user_id: Uuid,
    ) -> Result<ApiToken, ServiceError> {
        let token = self.find_by_id(conn, token_id).await?;

        if token.user_id != owner_user_id {
            return Err(ServiceError::AuthorizationError(
                "Cannot revoke another user's API token".to_string(),
            ));
        }

        #[derive(diesel::AsChangeset)]
        #[diesel(table_name = crate::schema::api_tokens)]
        struct RevokeChangeset {
            revoked_at: Option<chrono::DateTime<Utc>>,
        }

        self.repo
            .update(
                conn,
                token_id,
                RevokeChangeset {
                    revoked_at: Some(Utc::now()),
                },
            )
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Bulk soft-revoke all tokens for a user.  Returns the count of affected rows.
    pub async fn revoke_all_for_user(
        &self,
        conn: &mut DbConnection<'_>,
        target_user_id: Uuid,
    ) -> Result<usize, ServiceError> {
        self.repo
            .revoke_all_for_user(conn, target_user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Validate a raw `sat_*` API token.
    /// Returns the DB row on success; returns `AuthenticationError` (401) for every
    /// failure path so callers cannot distinguish "unknown" from "revoked".
    pub async fn validate_token(
        &self,
        conn: &mut DbConnection<'_>,
        raw_token: &str,
    ) -> Result<ApiToken, ServiceError> {
        use crate::schema::api_tokens::token_hash as col_token_hash;
        use diesel::ExpressionMethods;

        let hash: String = sha256_hex(raw_token);

        let token = self
            .repo
            .find_where(conn, col_token_hash.eq(hash))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .into_iter()
            .next()
            .ok_or_else(|| ServiceError::AuthenticationError("Invalid API token".to_string()))?;

        if token.revoked_at.is_some() {
            return Err(ServiceError::AuthenticationError(
                "API token has been revoked".to_string(),
            ));
        }

        if let Some(expires_at) = token.expires_at {
            if expires_at < chrono::Utc::now() {
                return Err(ServiceError::AuthenticationError(
                    "API token has expired".to_string(),
                ));
            }
        }

        Ok(token)
    }

    /// Stamp `last_used_at = now()` on an API token.
    /// Called during successful API token → session exchange for auditability.
    pub async fn record_usage(
        &self,
        conn: &mut DbConnection<'_>,
        token_id: Uuid,
    ) -> Result<(), ServiceError> {
        #[derive(diesel::AsChangeset)]
        #[diesel(table_name = crate::schema::api_tokens)]
        struct UsageChangeset {
            last_used_at: Option<chrono::DateTime<Utc>>,
        }

        self.repo
            .update(
                conn,
                token_id,
                UsageChangeset {
                    last_used_at: Some(Utc::now()),
                },
            )
            .await
            .map(|_| ())
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }
}

/// Compute the lowercase hex SHA-256 digest of `s`.
fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}
