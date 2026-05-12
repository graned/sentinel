//! DTOs for authentication endpoints: login, register, token lifecycle, and
//! the shared `AuthenticatedUserContext` inserted by `authenticate_middleware`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
//************* Login DTOs *************
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct BasicAuthLoginRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct BasicLoginResponse {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    /// True when the user was created by an admin and must change their temporary password.
    pub must_change_password: bool,
    /// True when admin has mandated MFA but the user has not yet enrolled TOTP.
    pub mfa_setup_required: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct AuthenticateRequest {
    pub access_token: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AuthContextResponse {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub roles: Vec<String>,
    pub email_verified: bool,
    pub must_change_password: bool,
    /// Present only for policy test tokens (scope = "policy_test").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// The policy ID embedded in a test token, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_test_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AuthenticatedUserContext {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub roles: Vec<String>,
    pub bypass_authorization: bool, // true only for API token auth
    pub email_verified: bool,
    pub must_change_password: bool,
}

//************* Refresh Token DTOs *************
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1))]
    pub refresh_token: String,
}

//************* Re-send Verification DTOs *************
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct ResendVerificationRequest {
    #[validate(email)]
    pub email: String,
}

//************* API Token Exchange DTOs *************
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct ExchangeApiTokenRequest {
    /// Email of the target user to create a session for.
    #[validate(email)]
    pub email: String,
}

//************* Verify email DTOs *************
/// Query parameters for the verify-email endpoint.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct VerifyEmailQuery {
    pub token: String,
}
