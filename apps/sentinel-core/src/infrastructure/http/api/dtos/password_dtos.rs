//! DTOs for password management endpoints: forgot/reset (public) and
//! change-password (authenticated).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::http::api::dtos::user_dtos::validate_password;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Must be a valid email address"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(custom = "validate_password")]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(custom = "validate_password")]
    pub new_password: String,
}
