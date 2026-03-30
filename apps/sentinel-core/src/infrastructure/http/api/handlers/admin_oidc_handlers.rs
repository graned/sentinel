//! HTTP handlers for admin OIDC configuration.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/admin/oidc/clients` | Register a new OIDC relying-party client |
//! | POST | `/admin/oidc/keys/generate` | Generate RSA signing key (retires previous active key) |
//! | GET  | `/admin/oidc/clients` | List all registered OIDC clients |

use crate::{
    http::api::dtos::{CreateOidcClientRequest, CreateOidcClientResponse, GenerateKeyResponse},
    http::api::routes::api_validation::ValidatedJson,
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};
use axum::Extension;
use std::sync::Arc;

/// POST /v1/api/admin/oidc/keys/generate
/// Generates a new RSA-2048 signing key for OIDC token signing and retires the previous active key.
#[utoipa::path(
    post,
    path = "/v1/api/admin/oidc/keys/generate",
    responses(
        (status = 200, description = "New signing key generated and activated", body = GenerateKeyResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "admin"
)]
pub async fn generate_signing_key(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<RawResponse<GenerateKeyResponse>, ApiError> {
    match state.oidc_application.generate_signing_key().await {
        Ok(res) => Ok(RawResponse(res)),
        Err(err) => Err(ApiError::from(err)),
    }
}

/// POST /v1/api/admin/oidc/clients
/// Registers a new OIDC client application.
#[utoipa::path(
    post,
    path = "/v1/api/admin/oidc/clients",
    request_body = CreateOidcClientRequest,
    responses(
        (status = 200, description = "OIDC client registered", body = CreateOidcClientResponse),
        (status = 400, description = "Validation error"),
    ),
    tag = "admin"
)]
pub async fn create_oidc_client(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(request): ValidatedJson<CreateOidcClientRequest>,
) -> Result<RawResponse<CreateOidcClientResponse>, ApiError> {
    match state.oidc_application.create_client(request).await {
        Ok(res) => Ok(RawResponse(res)),
        Err(err) => Err(ApiError::from(err)),
    }
}
