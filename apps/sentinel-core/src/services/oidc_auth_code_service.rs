//! OIDC authorization code service — creation and PKCE consumption.
//!
//! Authorization codes are short-lived, single-use tokens issued at the end of the
//! `GET /oauth/authorize` step and exchanged for JWTs at `POST /oauth/token`.
//!
//! # Code format
//!
//! 32 bytes from `OsRng` encoded as **URL-safe base64 without padding**.  Only the
//! SHA-256 hex digest is stored in `oidc_auth_codes.code_hash`.
//!
//! # Lifetime
//!
//! Codes expire **2 minutes** after creation (see `expires_at`).  The repository's
//! `consume_code` method sets `consumed_at` atomically and returns
//! `RepositoryError::NotFound` if the code is already consumed or expired.
//!
//! # PKCE (RFC 7636)
//!
//! At authorization time the client sends `code_challenge = BASE64URL(SHA256(verifier))`.
//! At token exchange time the client sends the raw `code_verifier`; this service
//! recomputes `BASE64URL(SHA256(verifier))` and checks it against the stored challenge.

use crate::{DbConnection, OidcAuthCode, OidcAuthCodeRepository, RepositoryError, ServiceError};
use base64::Engine;
use chrono::Utc;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

/// Manages OIDC authorization code issuance and PKCE verification.
pub struct OidcAuthCodeService {
    auth_code_repository: Arc<OidcAuthCodeRepository>,
}

impl OidcAuthCodeService {
    pub fn new(auth_code_repository: Arc<OidcAuthCodeRepository>) -> Self {
        Self {
            auth_code_repository,
        }
    }

    /// Generate and persist a new authorization code.
    ///
    /// Returns the **raw** code (not the hash) so it can be embedded in the redirect URL.
    /// The raw code must never be stored — only the SHA-256 hash is persisted.
    pub async fn create_code(
        &self,
        conn: &mut DbConnection<'_>,
        client_id: Uuid,
        user_id: Uuid,
        redirect_uri: &str,
        scope: &str,
        nonce: Option<&str>,
        code_challenge: &str,
        code_challenge_method: &str,
    ) -> Result<String, ServiceError> {
        // Generate 32 random bytes → URL-safe base64 (no padding) → raw code
        let mut raw_bytes = [0u8; 32];
        { rand::rngs::OsRng.fill_bytes(&mut raw_bytes); }
        let raw_code =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw_bytes);

        // Store SHA256(raw_code) as hex
        let code_hash = format!("{:x}", Sha256::digest(raw_code.as_bytes()));

        let auth_code = OidcAuthCode {
            oidc_auth_code_id: Uuid::new_v4(),
            code_hash,
            oidc_client_id: client_id,
            user_id,
            redirect_uri: redirect_uri.to_string(),
            scope: scope.to_string(),
            nonce: nonce.map(|s| s.to_string()),
            code_challenge: code_challenge.to_string(),
            code_challenge_method: code_challenge_method.to_string(),
            expires_at: Utc::now() + chrono::Duration::seconds(120), // 2 minutes
            consumed_at: None,
            created_at: Utc::now(),
        };

        self.auth_code_repository
            .create(conn, &auth_code)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(raw_code)
    }

    /// Validate and consume an authorization code.
    ///
    /// Steps performed:
    /// 1. Hash the raw code and look up the record (returns error if not found, expired, or consumed)
    /// 2. Verify the `client_id` matches the one stored in the code record
    /// 3. Verify the `redirect_uri` exactly matches the one used at authorization time
    /// 4. Verify the PKCE `code_verifier` against the stored `code_challenge`
    ///
    /// On success the code row is marked `consumed_at = now()` and the record is returned.
    pub async fn consume_code(
        &self,
        conn: &mut DbConnection<'_>,
        raw_code: &str,
        client_id: Uuid,
        redirect_uri: &str,
        code_verifier: &str,
    ) -> Result<OidcAuthCode, ServiceError> {
        // Compute code hash
        let code_hash = format!("{:x}", Sha256::digest(raw_code.as_bytes()));

        // Look up by code hash (returns error if already consumed or expired)
        let code_record = self
            .auth_code_repository
            .consume_code(conn, &code_hash)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    ServiceError::OidcInvalidCode("Authorization code is invalid, expired, or already consumed".to_string())
                }
                other => ServiceError::DatabaseError(other.to_string()),
            })?;

        // Validate client_id matches
        if code_record.oidc_client_id != client_id {
            return Err(ServiceError::OidcInvalidCode(
                "Authorization code was not issued to this client".to_string(),
            ));
        }

        // Validate redirect_uri exact match
        if code_record.redirect_uri != redirect_uri {
            return Err(ServiceError::OidcInvalidRedirectUri(
                "redirect_uri does not match the one used in the authorization request".to_string(),
            ));
        }

        // Verify PKCE: BASE64URL_NOPAD(SHA256(code_verifier)) == code_challenge
        if !self.verify_pkce(code_verifier, &code_record.code_challenge) {
            return Err(ServiceError::OidcPkceVerificationFailed(
                "PKCE code_verifier verification failed".to_string(),
            ));
        }

        Ok(code_record)
    }

    /// Verify the PKCE S256 challenge: `BASE64URL_NOPAD(SHA256(verifier)) == challenge`.
    fn verify_pkce(&self, code_verifier: &str, code_challenge: &str) -> bool {
        let hash = Sha256::digest(code_verifier.as_bytes());
        let computed =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash.as_slice());
        computed == code_challenge
    }
}
