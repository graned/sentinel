//! HTTP handlers for password management (`/v1/api/auth/password/*` and `/user/password/*`).
//!
//! # Public forgot/reset flow
//!
//! 1. `POST /auth/password/forgot` — user submits their email; always returns 200 to
//!    prevent email enumeration; if the email is found, sends a reset link.
//! 2. `POST /auth/password/reset` — user submits the `pr_*` token from the link plus a
//!    new password; token is consumed, all sessions revoked, notification email sent.
//!
//! # Authenticated change flow
//!
//! 3. `POST /user/password/change` — authenticated user submits current + new password;
//!    current password is verified, all sessions revoked, notification email sent.

use crate::{
    http::api::dtos::{
        AuthenticatedUserContext, ChangePasswordRequest, ForgotPasswordRequest,
        ResetPasswordRequest,
    },
    http::api::routes::api_validation::ValidatedJson,
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};
use axum::Extension;
use std::sync::Arc;

/// POST /v1/api/auth/password/forgot
/// Initiates the password reset flow. Always returns 200 to prevent email enumeration.
#[utoipa::path(
    post,
    path = "/v1/api/auth/password/forgot",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "If the email exists, a reset link has been sent"),
        (status = 400, description = "Validation error"),
    ),
    tag = "auth"
)]
pub async fn forgot_password(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<ForgotPasswordRequest>,
) -> Result<RawResponse<String>, ApiError> {
    state
        .auth_application
        .forgot_password(req.email)
        .await
        .map(|_| RawResponse("If that email is registered, a reset link has been sent".to_string()))
        .map_err(ApiError::from)
}

/// POST /v1/api/auth/password/reset
/// Consumes a reset token and sets a new password. Revokes all sessions.
#[utoipa::path(
    post,
    path = "/v1/api/auth/password/reset",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully"),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid, expired, or already-used token"),
    ),
    tag = "auth"
)]
pub async fn reset_password(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<ResetPasswordRequest>,
) -> Result<RawResponse<String>, ApiError> {
    state
        .auth_application
        .reset_password(req.token, req.new_password)
        .await
        .map(|_| RawResponse("Password reset successfully".to_string()))
        .map_err(ApiError::from)
}

/// POST /v1/api/user/password/change
/// Changes the password for the currently authenticated user. Requires valid Bearer token.
#[utoipa::path(
    post,
    path = "/v1/api/user/password/change",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully"),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid current password or missing Bearer token"),
    ),
    security(("BearerAuth" = [])),
    tag = "user"
)]
pub async fn change_password(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    ValidatedJson(req): ValidatedJson<ChangePasswordRequest>,
) -> Result<RawResponse<String>, ApiError> {
    state
        .user_password_application
        .change_password(ctx, req.current_password, req.new_password)
        .await
        .map(|_| RawResponse("Password changed successfully".to_string()))
        .map_err(ApiError::from)
}
