//! DTOs for user-profile endpoints (`/v1/api/user/*`) and shared types such as
//! `validate_password` (password policy validator) and `RoleResponse`.

use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::ValidationError;

use crate::UserStatus;

pub const MIN_PASSWORD_LEN: usize = 12;

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(ValidationError::new("password_too_short"));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::new("password_needs_uppercase"));
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ValidationError::new("password_needs_lowercase"));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("password_needs_digit"));
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ValidationError::new("password_needs_special"));
    }
    Ok(())
}

//************* Register User DTOs *************
#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct RegisterUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,

    #[validate(length(min = 1, max = 100))]
    pub last_name: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    pub avatar_url: Option<String>,

    #[validate(custom = "validate_password")]
    pub password: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct RegisterUserResponse {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserProfileResponse {
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub email: String,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize)]
pub struct AuthenticatedUserResponse {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub email: String,
    pub email_verified: bool,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserSessionResponse {
    pub session_id: Uuid,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub is_current: bool,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserSessionDetailResponse {
    pub session_id: Uuid,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub is_current: bool,
    pub is_active: bool,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct RoleResponse {
    pub role_id: Uuid,
    pub name: String,
    pub role_type: String,
    pub description: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserPermissionsResponse {
    pub user_id: Uuid,
    pub roles: Vec<RoleResponse>,
}
