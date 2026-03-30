//! DTOs for admin endpoints: role management, user administration, and
//! user auth-info inspection.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::UserStatus;

use super::user_dtos::{validate_password, RoleResponse};

#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct CreateRoleRequest {
    #[validate(custom(function = "validate_role_type"))]
    pub role_type: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 500))]
    pub description: String,
}

#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(min = 1))]
    pub description: Option<String>,
}

#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserAuthInfoResponse {
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub email: String,
    pub email_verified: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub roles: Vec<RoleResponse>,
}

fn validate_role_type(role_type: &str) -> Result<(), validator::ValidationError> {
    match role_type {
        "user" | "admin" | "support" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_role_type")),
    }
}

// ── Admin user management ────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AdminUserResponse {
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub email_verified: bool,
    pub status: UserStatus,
    pub roles: Vec<RoleResponse>,
    pub mfa_enabled: bool,
    pub mfa_required: bool,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct AdminSetMfaRequiredRequest {
    pub required: bool,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserMfaStatusResponse {
    pub mfa_required: bool,
    pub mfa_enabled: bool,
}

#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct AdminCreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    #[validate(custom(function = "validate_password"))]
    pub password: String,
    /// If true, a verification/invite email is sent to the user immediately.
    pub send_invite_email: Option<bool>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct InviteLinkResponse {
    /// The full verification URL the user must visit to confirm their email.
    pub invite_url: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateUserStatusRequest {
    /// Accepted values: "active", "suspended", "inactive"
    pub status: String,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct ListUsersQuery {
    /// Page number (1-based, default 1)
    pub page: Option<i64>,
    /// Items per page (default 20, max 100)
    pub page_size: Option<i64>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedUsersResponse {
    pub items: Vec<AdminUserResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
