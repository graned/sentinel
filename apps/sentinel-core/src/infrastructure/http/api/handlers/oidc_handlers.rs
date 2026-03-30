//! HTTP handlers for the OIDC / OAuth 2.0 endpoints.
//!
//! These handlers are mounted **outside** the standard API router so they bypass:
//! - The `ResponseWrapperLayer` (returns spec-compliant JSON / redirects, not Sentinel envelope)
//! - The `authenticate_middleware` on most routes (public discovery + JWKS)
//!
//! The `GET /oauth/authorize` endpoint *does* require a Sentinel Bearer token because
//! Sentinel acts as its own UI — the user is already logged in before the OIDC flow begins.
//!
//! | Method | Path | Auth | Description |
//! |--------|------|------|-------------|
//! | GET | `/.well-known/openid-configuration` | None | OIDC discovery document |
//! | GET | `/oauth/jwks.json` | None | Public key set (JWKS) for JWT verification |
//! | GET | `/oauth/authorize` | Bearer | Create auth code → redirect to client |
//! | POST | `/oauth/token` | Client credentials / PKCE | Exchange auth code for JWT tokens |

use crate::{
    http::api::dtos::{AuthenticatedUserContext, AuthorizeQuery, TokenExchangeForm, TokenResponse},
    http::server::AppState,
    ApiError,
};
use axum::{
    extract::{Form, Query},
    response::Redirect,
    Extension, Json,
};
use std::sync::Arc;

/// GET /.well-known/openid-configuration
/// Returns static OIDC discovery document (no auth required).
#[utoipa::path(
    get,
    path = "/.well-known/openid-configuration",
    responses(
        (status = 200, description = "OIDC discovery document",
            body = Object,
            content_type = "application/json",
            example = json!({
                "issuer": "https://auth.example.com",
                "authorization_endpoint": "https://auth.example.com/oauth/authorize",
                "token_endpoint": "https://auth.example.com/oauth/token",
                "jwks_uri": "https://auth.example.com/oauth/jwks.json",
                "response_types_supported": ["code"],
                "id_token_signing_alg_values_supported": ["RS256"]
            })
        ),
    ),
    tag = "oidc"
)]
pub async fn openid_configuration(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let issuer = state.oidc_application.issuer_url();
    Json(serde_json::json!({
        "issuer": issuer,
        "authorization_endpoint": format!("{}/oauth/authorize", issuer),
        "token_endpoint": format!("{}/oauth/token", issuer),
        "jwks_uri": format!("{}/oauth/jwks.json", issuer),
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "scopes_supported": ["openid", "email", "profile"],
        "token_endpoint_auth_methods_supported": ["client_secret_post", "none"],
        "code_challenge_methods_supported": ["S256"],
        "claims_supported": ["sub", "iss", "aud", "exp", "iat", "email", "nonce"]
    }))
}

/// GET /oauth/jwks.json
/// Returns JWKS public keys (no auth required).
#[utoipa::path(
    get,
    path = "/oauth/jwks.json",
    responses(
        (status = 200, description = "JSON Web Key Set (public signing keys)",
            body = Object,
            content_type = "application/json",
            example = json!({
                "keys": [{
                    "kty": "RSA",
                    "use": "sig",
                    "alg": "RS256",
                    "kid": "550e8400-e29b-41d4-a716-446655440000",
                    "n": "sLjV...",
                    "e": "AQAB"
                }]
            })
        ),
        (status = 500, description = "Internal server error"),
    ),
    tag = "oidc"
)]
pub async fn jwks(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    match state.oidc_application.get_jwks().await {
        Ok(keys) => Ok(Json(keys)),
        Err(err) => Err(ApiError::from(err)),
    }
}

/// GET /oauth/authorize
/// Validates OIDC params, creates an authorization code, and redirects to `redirect_uri`.
/// The user must have an active Sentinel session (Bearer PASETO token required).
#[utoipa::path(
    get,
    path = "/oauth/authorize",
    params(AuthorizeQuery),
    responses(
        (status = 303, description = "Redirect to redirect_uri with `code` and `state` query params"),
        (status = 400, description = "Invalid client, redirect URI, scope, or PKCE params"),
        (status = 401, description = "Missing or invalid Sentinel session token"),
    ),
    security(("BearerAuth" = [])),
    tag = "oidc"
)]
pub async fn authorize(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Query(params): Query<AuthorizeQuery>,
) -> Result<Redirect, ApiError> {
    match state.oidc_application.authorize(&params, &ctx).await {
        Ok(redirect_url) => Ok(Redirect::to(&redirect_url)),
        Err(err) => Err(ApiError::from(err)),
    }
}

/// POST /oauth/token
/// Exchanges an authorization code for JWT ID token and access token (PKCE flow).
/// No Sentinel session required — client authenticates via PKCE verifier or client_secret.
#[utoipa::path(
    post,
    path = "/oauth/token",
    request_body(
        content = TokenExchangeForm,
        content_type = "application/x-www-form-urlencoded",
        description = "Authorization code exchange parameters"
    ),
    responses(
        (status = 200, description = "JWT tokens issued", body = TokenResponse),
        (status = 400, description = "Invalid code, expired code, PKCE failure, or unsupported grant type"),
    ),
    tag = "oidc"
)]
pub async fn token_exchange(
    Extension(state): Extension<Arc<AppState>>,
    Form(form): Form<TokenExchangeForm>,
) -> Result<Json<TokenResponse>, ApiError> {
    match state.oidc_application.token_exchange(&form).await {
        Ok(response) => Ok(Json(response)),
        Err(err) => Err(ApiError::from(err)),
    }
}
