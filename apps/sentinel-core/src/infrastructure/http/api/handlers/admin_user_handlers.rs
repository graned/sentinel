//! HTTP handlers for the admin user-management endpoints (`/v1/api/admin/users/*`).
//!
//! All routes require the `admin` role.  The authorization check is enforced by
//! `authorize_middleware` via the policy engine — not in the handler code itself.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET    | `/admin/users` | List users with pagination |
//! | GET    | `/admin/users/{user_id}` | Get a single user |
//! | POST   | `/admin/users` | Create a user (admin-initiated) |
//! | PUT    | `/admin/users/{user_id}/status` | Enable / disable / set status |
//! | GET    | `/admin/users/{user_id}/auth-info` | Fetch MFA + identity details |
//! | GET    | `/admin/users/{user_id}/permissions` | List the user's roles |
//! | POST   | `/admin/users/{user_id}/mfa/disable` | Admin-force disable MFA |
//! | PUT    | `/admin/users/{user_id}/mfa-required` | Toggle admin-mandated MFA |
//! | GET    | `/admin/users/{user_id}/invite` | Generate a one-time invite link |

use crate::{
    http::api::dtos::{
        AdminCreateUserRequest, AdminUserResponse, AuthenticatedUserContext, InviteLinkResponse,
        ListUsersQuery, PaginatedUsersResponse, UpdateUserStatusRequest,
    },
    http::api::routes::api_validation::ValidatedJson,
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};
use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// GET /v1/api/admin/users
/// List users with server-side pagination. Requires admin role.
#[utoipa::path(
    get,
    path = "/v1/api/admin/users",
    params(ListUsersQuery),
    responses(
        (status = 200, description = "Paginated list of users", body = PaginatedUsersResponse),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn list_admin_users(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Query(query): Query<ListUsersQuery>,
) -> Result<RawResponse<PaginatedUsersResponse>, ApiError> {
    state
        .admin_application
        .list_users(&ctx, query)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// POST /v1/api/admin/users
/// Create (invite) a new user. Requires admin role.
#[utoipa::path(
    post,
    path = "/v1/api/admin/users",
    request_body = AdminCreateUserRequest,
    responses(
        (status = 200, description = "User created", body = AdminUserResponse),
        (status = 400, description = "Validation error or email already in use"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn create_admin_user(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    ValidatedJson(req): ValidatedJson<AdminCreateUserRequest>,
) -> Result<RawResponse<AdminUserResponse>, ApiError> {
    state
        .admin_application
        .create_user(&ctx, req)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// DELETE /v1/api/admin/users/{user_id}
/// Delete a user by ID. Requires admin role.
#[utoipa::path(
    delete,
    path = "/v1/api/admin/users/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "UUID of the user to delete"),
    ),
    responses(
        (status = 200, description = "User deleted"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "User not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn delete_admin_user(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(user_id): Path<Uuid>,
) -> Result<RawResponse<()>, ApiError> {
    state
        .admin_application
        .delete_user(&ctx, user_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// POST /v1/api/admin/users/{user_id}/send-invite
/// Send an invite/verification email to an admin-created user. Requires admin role.
#[utoipa::path(
    post,
    path = "/v1/api/admin/users/{user_id}/send-invite",
    params(
        ("user_id" = Uuid, Path, description = "UUID of the user to invite"),
    ),
    responses(
        (status = 200, description = "Invite email sent"),
        (status = 400, description = "Email already verified"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "User not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn send_user_invite(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(user_id): Path<Uuid>,
) -> Result<RawResponse<()>, ApiError> {
    state
        .admin_application
        .send_invite(&ctx, user_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// GET /v1/api/admin/users/{user_id}/invite-link
/// Generate an invite link for an admin-created user without sending an email. Requires admin role.
#[utoipa::path(
    get,
    path = "/v1/api/admin/users/{user_id}/invite-link",
    params(
        ("user_id" = Uuid, Path, description = "UUID of the user"),
    ),
    responses(
        (status = 200, description = "Invite link generated", body = InviteLinkResponse),
        (status = 400, description = "Email already verified"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "User not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn get_user_invite_link(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(user_id): Path<Uuid>,
) -> Result<RawResponse<InviteLinkResponse>, ApiError> {
    state
        .admin_application
        .get_invite_link(&ctx, user_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// PUT /v1/api/admin/users/{user_id}/status
/// Update a user's status (active / suspended / inactive). Requires admin role.
#[utoipa::path(
    put,
    path = "/v1/api/admin/users/{user_id}/status",
    params(
        ("user_id" = Uuid, Path, description = "UUID of the user"),
    ),
    request_body = UpdateUserStatusRequest,
    responses(
        (status = 200, description = "User status updated", body = AdminUserResponse),
        (status = 400, description = "Invalid status value"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "User not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn update_admin_user_status(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserStatusRequest>,
) -> Result<RawResponse<AdminUserResponse>, ApiError> {
    state
        .admin_application
        .update_user_status(&ctx, user_id, req)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}
