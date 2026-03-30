//! OIDC application layer — implements Sentinel as an OpenID Connect Identity Provider.
//!
//! Sentinel supports the **Authorization Code + PKCE** flow only.
//! External applications ("relying parties") register as OIDC clients and can issue
//! "Log in with Sentinel" flows to their users.
//!
//! # Flow summary
//!
//! ```text
//! 1. Client redirects user to:
//!    GET /oauth/authorize?response_type=code&client_id=…&redirect_uri=…&code_challenge=…
//!    → Sentinel validates params, creates an auth code, returns redirect to client
//!
//! 2. Client exchanges the code:
//!    POST /oauth/token  (application/x-www-form-urlencoded)
//!    grant_type=authorization_code&code=…&code_verifier=…
//!    → PKCE verification → Sentinel issues RS256 JWT id_token + JWT access_token
//! ```
//!
//! # Token format
//!
//! OIDC tokens are **JWTs signed with RS256** (not PASETO) to comply with the spec.
//! The RSA signing key is stored encrypted in `oidc_signing_keys` and must be
//! generated via `POST /v1/api/admin/oidc/keys/generate` before any flows work.
//!
//! # `amr` claim
//!
//! The `amr` (Authentication Methods References) claim in the ID token reflects
//! how the user authenticated: `["pwd"]` for password-only, `["pwd", "totp"]` when MFA was used.

use crate::{
    http::api::dtos::{
        AuthenticatedUserContext, AuthorizeQuery, CreateOidcClientRequest,
        CreateOidcClientResponse, GenerateKeyResponse, TokenExchangeForm, TokenResponse,
    },
    IdentityService, MfaTotpService, OidcAuthCodeService, OidcClient, OidcClientService,
    OidcKeyService, OidcTokenService, PostgresClient, ServiceError, UserService,
};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

pub struct OidcApplication {
    pg_client: Arc<PostgresClient>,
    client_service: Arc<OidcClientService>,
    auth_code_service: Arc<OidcAuthCodeService>,
    key_service: Arc<OidcKeyService>,
    token_service: Arc<OidcTokenService>,
    user_service: Arc<UserService>,
    identity_service: Arc<IdentityService>,
    mfa_totp_service: Arc<MfaTotpService>,
}

impl OidcApplication {
    pub fn new(
        pg_client: Arc<PostgresClient>,
        client_service: Arc<OidcClientService>,
        auth_code_service: Arc<OidcAuthCodeService>,
        key_service: Arc<OidcKeyService>,
        token_service: Arc<OidcTokenService>,
        user_service: Arc<UserService>,
        identity_service: Arc<IdentityService>,
        mfa_totp_service: Arc<MfaTotpService>,
    ) -> Self {
        Self {
            pg_client,
            client_service,
            auth_code_service,
            key_service,
            token_service,
            user_service,
            identity_service,
            mfa_totp_service,
        }
    }

    pub fn issuer_url(&self) -> String {
        self.token_service.issuer_url().to_string()
    }

    /// OIDC Authorization endpoint: validate params, create auth code, return redirect URL.
    pub async fn authorize(
        &self,
        params: &AuthorizeQuery,
        user_ctx: &AuthenticatedUserContext,
    ) -> Result<String, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        // Validate response_type
        if params.response_type != "code" {
            return Err(ServiceError::OidcInvalidCode(
                "Only response_type=code is supported".to_string(),
            ));
        }

        // Validate code_challenge_method
        if params.code_challenge_method != "S256" {
            return Err(ServiceError::OidcPkceVerificationFailed(
                "Only S256 code_challenge_method is supported".to_string(),
            ));
        }

        // Validate code_challenge format (RFC 7636: 43–128 chars, base64url alphabet)
        if !is_valid_pkce_challenge(&params.code_challenge) {
            return Err(ServiceError::OidcPkceVerificationFailed(
                "Invalid code_challenge format".to_string(),
            ));
        }

        // Look up client
        let client = self
            .client_service
            .find_by_client_id(&mut conn, &params.client_id)
            .await?;

        // Validate redirect_uri
        self.client_service
            .validate_redirect_uri(&client, &params.redirect_uri)?;

        // Validate scopes
        self.client_service
            .validate_scopes(&client, &params.scope)?;

        // Create auth code
        let raw_code = self
            .auth_code_service
            .create_code(
                &mut conn,
                client.oidc_client_id,
                user_ctx.user_id,
                &params.redirect_uri,
                &params.scope,
                params.nonce.as_deref(),
                &params.code_challenge,
                &params.code_challenge_method,
            )
            .await?;

        // Build redirect URL
        let redirect_url = format!(
            "{}?code={}&state={}",
            params.redirect_uri, raw_code, params.state
        );

        Ok(redirect_url)
    }

    /// Token exchange: validate code + PKCE, issue JWT tokens.
    pub async fn token_exchange(
        &self,
        form: &TokenExchangeForm,
    ) -> Result<TokenResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        if form.grant_type != "authorization_code" {
            return Err(ServiceError::OidcInvalidCode(
                "Only grant_type=authorization_code is supported".to_string(),
            ));
        }

        // Look up client
        let client = self
            .client_service
            .find_by_client_id(&mut conn, &form.client_id)
            .await?;

        // Validate client secret for confidential clients
        if client.is_confidential {
            if let Some(secret) = &form.client_secret {
                self.client_service
                    .validate_client_secret(&client, secret)?;
            } else {
                return Err(ServiceError::OidcInvalidCode(
                    "client_secret is required for confidential clients".to_string(),
                ));
            }
        }

        // Consume the auth code (validates redirect_uri + PKCE + expiry + consumed)
        let code_record = self
            .auth_code_service
            .consume_code(
                &mut conn,
                &form.code,
                client.oidc_client_id,
                &form.redirect_uri,
                &form.code_verifier,
            )
            .await?;

        // Fetch user
        let user = self
            .user_service
            .find_user_by_id(&mut conn, code_record.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User not found".to_string()))?;

        // Fetch primary identity (for email claim)
        let identity = self
            .identity_service
            .find_primary_identity_by_user_id(&mut conn, code_record.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Identity not found".to_string()))?;

        // Get active signing key
        let active_key = self.key_service.get_active_key(&mut conn).await?;

        // Decrypt private key DER
        let private_der = self
            .key_service
            .decrypt_private_key(&active_key.private_key_encrypted)?;

        let exp_secs = 3600u64; // 1 hour

        // Build AMR claim based on whether MFA is enabled for this user
        let mfa_enabled = self
            .mfa_totp_service
            .is_mfa_enabled(&mut conn, user.user_id)
            .await?;
        let amr = Some(if mfa_enabled {
            vec!["pwd".to_string(), "totp".to_string()]
        } else {
            vec!["pwd".to_string()]
        });

        // Generate ID token
        let id_token = self.token_service.generate_id_token(
            &active_key,
            &private_der,
            &user,
            &identity,
            &form.client_id,
            code_record.nonce.as_deref(),
            exp_secs,
            amr,
        )?;

        // Generate access token
        let access_token = self.token_service.generate_access_token(
            &active_key,
            &private_der,
            user.user_id,
            &form.client_id,
            &code_record.scope,
            exp_secs,
        )?;

        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: exp_secs,
            id_token,
            scope: code_record.scope,
        })
    }

    /// Admin: generate a new RSA signing key (retires previous active key).
    pub async fn generate_signing_key(&self) -> Result<GenerateKeyResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        let key = self.key_service.generate_and_store_key(&mut conn).await?;
        Ok(GenerateKeyResponse {
            kid: key.kid,
            alg: key.alg,
            status: key.status,
            created_at: key.created_at,
        })
    }

    /// Admin: create a new OIDC client.
    pub async fn create_client(
        &self,
        request: CreateOidcClientRequest,
    ) -> Result<CreateOidcClientResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        let client_secret_hash = request
            .client_secret
            .as_ref()
            .map(|s| format!("{:x}", Sha256::digest(s.as_bytes())));

        let client = OidcClient {
            oidc_client_id: Uuid::new_v4(),
            tenant_id: None,
            client_id: request.client_id,
            client_secret_hash,
            name: request.name,
            redirect_uris: request.redirect_uris,
            allowed_scopes: request.allowed_scopes,
            pkce_required: request.pkce_required,
            is_confidential: request.is_confidential,
            created_at: Utc::now(),
        };

        let stored = self
            .client_service
            .create_client(&mut conn, &client)
            .await?;

        Ok(CreateOidcClientResponse {
            oidc_client_id: stored.oidc_client_id,
            client_id: stored.client_id,
            name: stored.name,
            redirect_uris: stored.redirect_uris,
            allowed_scopes: stored.allowed_scopes,
            is_confidential: stored.is_confidential,
            pkce_required: stored.pkce_required,
            created_at: stored.created_at,
        })
    }

    /// Get the JWKS (public keys) for token verification.
    pub async fn get_jwks(&self) -> Result<serde_json::Value, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        self.key_service.get_jwks(&mut conn).await
    }
}

/// Validate that a PKCE code_challenge is a valid base64url string of 43–128 characters (RFC 7636).
fn is_valid_pkce_challenge(s: &str) -> bool {
    s.len() >= 43
        && s.len() <= 128
        && s.chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '.' | '_' | '~'))
}
