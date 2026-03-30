//! HTTP handlers for admin session management.
//!
//! Allows admins to view and force-revoke active sessions across all users.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET    | `/admin/sessions` | List all active sessions (non-revoked, non-expired) with user email |
//! | GET    | `/admin/sessions/{session_id}` | Get a specific session |
//! | POST   | `/admin/sessions/revoke` | Bulk-revoke a list of sessions by ID |
//! | DELETE | `/admin/sessions/{session_id}` | Revoke a single session |

use crate::{
    http::api::dtos::{
        AdminSessionResponse, AuthenticatedUserContext, BulkRevokeSessionsRequest,
        BulkRevokeSessionsResponse,
    },
    http::api::routes::api_validation::ValidatedJson,
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};
use axum::{extract::Path, Extension};
use std::sync::Arc;
use uuid::Uuid;

/// GET /v1/api/admin/sessions
/// List all active sessions across all users. Requires admin role.
#[utoipa::path(
    get,
    path = "/v1/api/admin/sessions",
    responses(
        (status = 200, description = "List of all active sessions", body = Vec<AdminSessionResponse>),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn get_all_admin_sessions(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<Vec<AdminSessionResponse>>, ApiError> {
    state
        .admin_session_application
        .get_all_active_sessions(ctx)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// DELETE /v1/api/admin/sessions/{session_id}
/// Invalidate a single session by ID. Requires admin role.
#[utoipa::path(
    delete,
    path = "/v1/api/admin/sessions/{session_id}",
    params(
        ("session_id" = Uuid, Path, description = "Session ID to invalidate"),
    ),
    responses(
        (status = 200, description = "Session invalidated"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Session not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn revoke_admin_session(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(session_id): Path<Uuid>,
) -> Result<RawResponse<()>, ApiError> {
    state
        .admin_session_application
        .revoke_session(ctx, session_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// POST /v1/api/admin/sessions/revoke
/// Bulk-invalidate sessions by ID list. Requires admin role.
#[utoipa::path(
    post,
    path = "/v1/api/admin/sessions/revoke",
    request_body = BulkRevokeSessionsRequest,
    responses(
        (status = 200, description = "Sessions invalidated", body = BulkRevokeSessionsResponse),
        (status = 400, description = "Validation error — session_ids must not be empty"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn revoke_admin_sessions_bulk(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    ValidatedJson(body): ValidatedJson<BulkRevokeSessionsRequest>,
) -> Result<RawResponse<BulkRevokeSessionsResponse>, ApiError> {
    state
        .admin_session_application
        .revoke_sessions_bulk(ctx, body)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}
