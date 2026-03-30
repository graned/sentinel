//! HTTP handlers for long-lived API token management.
//!
//! API tokens (`sat_<hex>`) allow programmatic access without a full login flow.
//! All endpoints require an admin Bearer token.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST   | `/auth/api-tokens` | Create a token — raw value returned exactly once |
//! | GET    | `/auth/api-tokens` | List the caller's tokens (no raw values) |
//! | DELETE | `/auth/api-tokens/{id}` | Soft-revoke one token |
//! | DELETE | `/auth/api-tokens` | Soft-revoke all tokens for the caller |

use crate::{
    http::api::dtos::{
        ApiTokenResponse, AuthenticatedUserContext, CreateApiTokenRequest, CreateApiTokenResponse,
    },
    http::api::routes::api_validation::ValidatedJson,
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};
use axum::{extract::Path, Extension};
use std::sync::Arc;
use uuid::Uuid;

/// Create a new long-lived API token for the authenticated admin user.
/// The raw token is returned exactly once — store it securely.
#[utoipa::path(
    post,
    path = "/v1/api/auth/api-tokens",
    request_body = CreateApiTokenRequest,
    responses(
        (status = 200, description = "API token created", body = CreateApiTokenResponse),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "auth"
)]
pub async fn create_api_token(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    ValidatedJson(request): ValidatedJson<CreateApiTokenRequest>,
) -> Result<RawResponse<CreateApiTokenResponse>, ApiError> {
    state
        .api_token_application
        .create_api_token(ctx, request)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// List all API tokens belonging to the authenticated admin user.
#[utoipa::path(
    get,
    path = "/v1/api/auth/api-tokens",
    responses(
        (status = 200, description = "List of API tokens", body = Vec<ApiTokenResponse>),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "auth"
)]
pub async fn list_api_tokens(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<Vec<ApiTokenResponse>>, ApiError> {
    state
        .api_token_application
        .list_api_tokens(ctx)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// Revoke a specific API token by its ID.
#[utoipa::path(
    delete,
    path = "/v1/api/auth/api-tokens/{token_id}",
    params(
        ("token_id" = Uuid, Path, description = "UUID of the API token to revoke"),
    ),
    responses(
        (status = 200, description = "Token revoked", body = String),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Token not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "auth"
)]
pub async fn revoke_api_token(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(token_id): Path<Uuid>,
) -> Result<RawResponse<String>, ApiError> {
    state
        .api_token_application
        .revoke_api_token(ctx, token_id)
        .await
        .map(|_| RawResponse("revoked".to_string()))
        .map_err(ApiError::from)
}

/// Revoke all API tokens belonging to the authenticated admin user.
#[utoipa::path(
    delete,
    path = "/v1/api/auth/api-tokens",
    responses(
        (status = 200, description = "All tokens revoked", body = String),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "auth"
)]
pub async fn revoke_all_tokens(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<String>, ApiError> {
    state
        .api_token_application
        .revoke_all_api_tokens(ctx)
        .await
        .map(|_| RawResponse("revoked all".to_string()))
        .map_err(ApiError::from)
}
