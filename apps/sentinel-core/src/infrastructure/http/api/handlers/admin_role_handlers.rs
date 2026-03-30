//! HTTP handlers for admin role management and user–role assignments.
//!
//! All routes require the `admin` role via the policy engine.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST   | `/admin/roles` | Create a new role |
//! | GET    | `/admin/roles` | List all roles |
//! | PUT    | `/admin/roles/{id}` | Update role name/description |
//! | DELETE | `/admin/roles/{id}` | Delete a role |
//! | POST   | `/admin/users/{user_id}/roles` | Assign a role to a user |
//! | DELETE | `/admin/users/{user_id}/roles/{role_name}` | Remove a role from a user |
//! | GET    | `/admin/users/{user_id}/permissions` | List roles assigned to a user |
//! | GET    | `/admin/users/{user_id}/auth-info` | Get MFA status + identity details |

use crate::{
    http::api::dtos::{
        AdminSetMfaRequiredRequest, AssignRoleRequest, AuthenticatedUserContext, CreateRoleRequest,
        RoleResponse, UpdateRoleRequest, UserAuthInfoResponse, UserMfaStatusResponse,
        UserPermissionsResponse,
    },
    http::api::routes::api_validation::ValidatedJson,
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};
use axum::{extract::Path, Extension};
use std::sync::Arc;
use uuid::Uuid;

/// POST /v1/api/admin/roles
/// Create a new role. Requires admin role.
#[utoipa::path(
    post,
    path = "/v1/api/admin/roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 200, description = "Role created", body = RoleResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn create_role(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    ValidatedJson(req): ValidatedJson<CreateRoleRequest>,
) -> Result<RawResponse<RoleResponse>, ApiError> {
    state
        .admin_application
        .create_role(&ctx, req)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// GET /v1/api/admin/roles
/// List all roles. Requires admin role.
#[utoipa::path(
    get,
    path = "/v1/api/admin/roles",
    responses(
        (status = 200, description = "List of roles", body = Vec<RoleResponse>),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn list_roles(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<Vec<RoleResponse>>, ApiError> {
    state
        .admin_application
        .list_roles(&ctx)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// PUT /v1/api/admin/roles/{role_id}
/// Update an existing role. Requires admin role.
#[utoipa::path(
    put,
    path = "/v1/api/admin/roles/{role_id}",
    params(
        ("role_id" = Uuid, Path, description = "UUID of the role to update"),
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated", body = RoleResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Role not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn update_role(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(role_id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateRoleRequest>,
) -> Result<RawResponse<RoleResponse>, ApiError> {
    state
        .admin_application
        .update_role(&ctx, role_id, req)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// DELETE /v1/api/admin/roles/{role_id}
/// Delete a role. Requires admin role.
#[utoipa::path(
    delete,
    path = "/v1/api/admin/roles/{role_id}",
    params(
        ("role_id" = Uuid, Path, description = "UUID of the role to delete"),
    ),
    responses(
        (status = 200, description = "Role deleted"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Role not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn delete_role(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(role_id): Path<Uuid>,
) -> Result<RawResponse<()>, ApiError> {
    state
        .admin_application
        .delete_role(&ctx, role_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// POST /v1/api/admin/users/{user_id}/roles
/// Assign a role to a user. Requires admin role.
#[utoipa::path(
    post,
    path = "/v1/api/admin/users/{user_id}/roles",
    params(
        ("user_id" = Uuid, Path, description = "UUID of the user"),
    ),
    request_body = AssignRoleRequest,
    responses(
        (status = 200, description = "Role assigned", body = RoleResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "User or role not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn assign_role_to_user(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(user_id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<AssignRoleRequest>,
) -> Result<RawResponse<RoleResponse>, ApiError> {
    state
        .admin_application
        .assign_role_to_user(&ctx, user_id, req)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// DELETE /v1/api/admin/users/{user_id}/roles/{role_name}
/// Remove a role from a user. Requires admin role.
#[utoipa::path(
    delete,
    path = "/v1/api/admin/users/{user_id}/roles/{role_name}",
    params(
        ("user_id" = Uuid, Path, description = "UUID of the user"),
        ("role_name" = String, Path, description = "Name of the role to remove"),
    ),
    responses(
        (status = 200, description = "Role removed"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "User or role not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn remove_role_from_user(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path((user_id, role_name)): Path<(Uuid, String)>,
) -> Result<RawResponse<()>, ApiError> {
    state
        .admin_application
        .remove_role_from_user(&ctx, user_id, role_name)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// GET /v1/api/admin/users/{user_id}/permissions
/// Get permissions (roles) for a user. Requires admin role.
#[utoipa::path(
    get,
    path = "/v1/api/admin/users/{user_id}/permissions",
    params(
        ("user_id" = Uuid, Path, description = "UUID of the user"),
    ),
    responses(
        (status = 200, description = "User permissions", body = UserPermissionsResponse),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "User not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn get_user_permissions_admin(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(user_id): Path<Uuid>,
) -> Result<RawResponse<UserPermissionsResponse>, ApiError> {
    state
        .admin_application
        .get_user_permissions(&ctx, user_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// PUT /v1/api/admin/users/{user_id}/mfa
/// Set MFA required flag for a user. Requires admin role.
/// If enabled, all existing sessions for the user are revoked immediately.
#[utoipa::path(
    put,
    path = "/v1/api/admin/users/{user_id}/mfa",
    params(
        ("user_id" = Uuid, Path, description = "UUID of the user"),
    ),
    request_body = AdminSetMfaRequiredRequest,
    responses(
        (status = 200, description = "MFA requirement updated", body = UserMfaStatusResponse),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "User not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn set_user_mfa_required(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(user_id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<AdminSetMfaRequiredRequest>,
) -> Result<RawResponse<UserMfaStatusResponse>, ApiError> {
    state
        .admin_application
        .set_mfa_required(&ctx, user_id, req)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// GET /v1/api/admin/users/{user_id}/auth-info
/// Get full auth info for a user. Requires admin role.
#[utoipa::path(
    get,
    path = "/v1/api/admin/users/{user_id}/auth-info",
    params(
        ("user_id" = Uuid, Path, description = "UUID of the user"),
    ),
    responses(
        (status = 200, description = "User auth info", body = UserAuthInfoResponse),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "User not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn get_user_auth_info(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(user_id): Path<Uuid>,
) -> Result<RawResponse<UserAuthInfoResponse>, ApiError> {
    state
        .admin_application
        .get_user_auth_info(&ctx, user_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}
