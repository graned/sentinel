//! HTTP handlers for the `/v1/api/user/*` route group.
//!
//! All routes in this file require a valid Bearer token (enforced by
//! `authenticate_middleware`).  The `AuthenticatedUserContext` extractor provides
//! the current user's `user_id`, `roles`, etc. without additional DB queries.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/user/me` | Return the current user's profile |
//! | PATCH | `/user/me` | Update the current user's profile |
//! | GET | `/user/sessions` | List all sessions (paginated) |
//! | GET | `/user/sessions/{session_id}` | Get a specific session |
//! | GET | `/user/permissions` | List the current user's roles/permissions |
//! | GET | `/user/canary` | Auth + authz demo endpoint |

use crate::{
    http::api::dtos::{
        AuthenticatedUserContext, UpdateProfileRequest, UserPermissionsResponse, UserProfileResponse,
        UserSessionDetailResponse, UserSessionResponse,
    },
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};
use axum::{extract::Path, Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/v1/api/user/me",
    responses(
        (status = 200, description = "Authenticated user's profile", body = UserProfileResponse),
        (status = 401, description = "Missing or invalid token"),
    ),
    security(("BearerAuth" = [])),
    tag = "user"
)]
pub async fn get_me(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<UserProfileResponse>, ApiError> {
    state
        .user_application
        .get_me(ctx)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    patch,
    path = "/v1/api/user/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Updated user profile", body = UserProfileResponse),
        (status = 401, description = "Missing or invalid token"),
    ),
    security(("BearerAuth" = [])),
    tag = "user"
)]
pub async fn update_me(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<RawResponse<UserProfileResponse>, ApiError> {
    state
        .user_application
        .update_me(ctx, req.first_name, req.last_name, req.avatar_url)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/v1/api/user/sessions",
    responses(
        (status = 200, description = "Active sessions for the authenticated user",
         body = Vec<UserSessionResponse>),
        (status = 401, description = "Missing or invalid token"),
    ),
    security(("BearerAuth" = [])),
    tag = "user"
)]
pub async fn get_user_sessions(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<Vec<UserSessionResponse>>, ApiError> {
    state
        .user_application
        .get_sessions(ctx)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/v1/api/user/sessions/{session_id}",
    params(("session_id" = Uuid, Path, description = "Session UUID")),
    responses(
        (status = 200, description = "Session details", body = UserSessionDetailResponse),
        (status = 401, description = "Missing or invalid token"),
        (status = 404, description = "Session not found or not owned by user"),
    ),
    security(("BearerAuth" = [])),
    tag = "user"
)]
pub async fn get_user_session(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(session_id): Path<Uuid>,
) -> Result<RawResponse<UserSessionDetailResponse>, ApiError> {
    state
        .user_application
        .get_session(ctx, session_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/v1/api/user/permissions",
    responses(
        (status = 200, description = "User roles and permissions", body = UserPermissionsResponse),
        (status = 401, description = "Missing or invalid token"),
    ),
    security(("BearerAuth" = [])),
    tag = "user"
)]
pub async fn get_user_permissions(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<UserPermissionsResponse>, ApiError> {
    state
        .user_application
        .get_permissions(ctx)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}
