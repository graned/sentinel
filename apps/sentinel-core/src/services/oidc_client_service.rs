//! OIDC client service — registration and validation of OAuth 2.0 / OIDC relying parties.
//!
//! An **OIDC client** represents an external application that uses Sentinel as its
//! identity provider (a "relying party").  Clients are registered by admins and stored
//! in the `oidc_clients` table.
//!
//! # Client types
//!
//! | Type | `is_confidential` | Secret required? |
//! |------|-------------------|-----------------|
//! | Public | `false` | No — PKCE is the only proof |
//! | Confidential | `true` | Yes — `client_secret` must be sent with the token request |
//!
//! For confidential clients, only the SHA-256 hash of the secret is stored.
//! Validation uses `subtle::ConstantTimeEq` to prevent timing attacks.

use crate::{DbConnection, OidcClient, OidcClientRepository, ServiceError};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use subtle::ConstantTimeEq;

/// Handles OIDC client lookup and parameter validation.
pub struct OidcClientService {
    client_repository: Arc<OidcClientRepository>,
}

impl OidcClientService {
    pub fn new(client_repository: Arc<OidcClientRepository>) -> Self {
        Self { client_repository }
    }

    pub async fn find_by_client_id(
        &self,
        conn: &mut DbConnection<'_>,
        client_id: &str,
    ) -> Result<OidcClient, ServiceError> {
        self.client_repository
            .find_by_client_id(conn, client_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                ServiceError::OidcClientNotFound(format!("OIDC client '{}' not found", client_id))
            })
    }

    /// Check that `uri` is an exact match to one of the client's registered `redirect_uris`.
    /// Exact matching (not prefix) is required by RFC 6749 § 3.1.2.3.
    pub fn validate_redirect_uri(
        &self,
        client: &OidcClient,
        uri: &str,
    ) -> Result<(), ServiceError> {
        if client.redirect_uris.iter().any(|r| r == uri) {
            Ok(())
        } else {
            Err(ServiceError::OidcInvalidRedirectUri(format!(
                "Redirect URI '{}' is not registered for this client",
                uri
            )))
        }
    }

    /// Verify that all space-separated scopes in `requested` are listed in the client's
    /// `allowed_scopes`.  The `openid` scope is mandatory for OIDC flows.
    pub fn validate_scopes(
        &self,
        client: &OidcClient,
        requested: &str,
    ) -> Result<(), ServiceError> {
        let requested_scopes: Vec<&str> = requested.split_whitespace().collect();

        if !requested_scopes.contains(&"openid") {
            return Err(ServiceError::OidcInvalidScope(
                "Scope must include 'openid'".to_string(),
            ));
        }

        for scope in &requested_scopes {
            if !client.allowed_scopes.iter().any(|s| s == scope) {
                return Err(ServiceError::OidcInvalidScope(format!(
                    "Scope '{}' is not allowed for this client",
                    scope
                )));
            }
        }

        Ok(())
    }

    /// Compare the presented `secret` against the stored SHA-256 hash using
    /// constant-time comparison to prevent timing attacks.
    pub fn validate_client_secret(
        &self,
        client: &OidcClient,
        secret: &str,
    ) -> Result<(), ServiceError> {
        let hash = format!("{:x}", Sha256::digest(secret.as_bytes()));
        match &client.client_secret_hash {
            Some(stored_hash) => {
                let ok: bool = stored_hash.as_bytes().ct_eq(hash.as_bytes()).into();
                if ok {
                    Ok(())
                } else {
                    Err(ServiceError::OidcInvalidCode("Invalid client secret".to_string()))
                }
            }
            None => Err(ServiceError::OidcInvalidCode(
                "Invalid client secret".to_string(),
            )),
        }
    }

    pub async fn list_all_clients(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<Vec<OidcClient>, ServiceError> {
        self.client_repository
            .find_all(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn create_client(
        &self,
        conn: &mut DbConnection<'_>,
        client: &OidcClient,
    ) -> Result<OidcClient, ServiceError> {
        self.client_repository
            .create(conn, client)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }
}
