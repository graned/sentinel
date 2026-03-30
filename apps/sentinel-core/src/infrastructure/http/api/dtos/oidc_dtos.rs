//! DTOs for OIDC / OAuth 2.0 endpoints: authorization request, token exchange,
//! OIDC client creation, and signing-key management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

/// GET /oauth/authorize query params
#[derive(Debug, Deserialize, IntoParams)]
pub struct AuthorizeQuery {
    /// Must be "code"
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    /// Space-separated scopes (e.g. "openid email")
    pub scope: String,
    pub state: String,
    pub nonce: Option<String>,
    /// BASE64URL(SHA256(code_verifier))
    pub code_challenge: String,
    /// Must be "S256"
    pub code_challenge_method: String,
}

/// POST /oauth/token form body (application/x-www-form-urlencoded)
#[derive(Debug, Deserialize, ToSchema)]
pub struct TokenExchangeForm {
    /// Must be "authorization_code"
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub code_verifier: String,
    pub client_secret: Option<String>,
}

/// Token response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub id_token: String,
    pub scope: String,
}

/// Admin: create OIDC client request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateOidcClientRequest {
    #[validate(length(min = 1))]
    pub client_id: String,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub is_confidential: bool,
    pub pkce_required: bool,
    pub client_secret: Option<String>,
}

/// Admin: create OIDC client response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOidcClientResponse {
    pub oidc_client_id: Uuid,
    pub client_id: String,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub is_confidential: bool,
    pub pkce_required: bool,
    pub created_at: DateTime<Utc>,
}

/// Admin: generate signing key response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateKeyResponse {
    pub kid: String,
    pub alg: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
