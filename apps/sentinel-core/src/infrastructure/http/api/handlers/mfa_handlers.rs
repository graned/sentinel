//! HTTP handlers for MFA (TOTP) enrollment and verification.
//!
//! # Enrollment flow
//!
//! 1. `POST /auth/mfa/totp/start` → `mfa_totp_start`
//!    Requires a valid Bearer session.  Returns a `otpauth://` URI the user scans with
//!    their authenticator app.  The secret is stored encrypted but MFA is not yet active.
//!
//! 2. `POST /auth/mfa/totp/confirm` → `mfa_totp_confirm`
//!    Requires the same Bearer session.  The user submits the first TOTP code to prove
//!    they can generate codes.  On success, `user_mfa_totp.enabled` is set to `true`.
//!
//! # Login with MFA
//!
//! 3. `POST /auth/mfa/verify` → `mfa_verify`
//!    Does **not** require a Bearer token.  Instead, the client sends the short-lived
//!    `mfa_session_token` (PASETO, 5-min TTL) obtained from `basic_auth_login` alongside
//!    a TOTP code (or recovery code).  On success, returns full session tokens.

use crate::{
    http::api::dtos::{
        AuthenticatedUserContext, BasicLoginResponse, ConfirmMfaEnrollmentRequest,
        ConfirmMfaEnrollmentResponse, StartMfaEnrollmentResponse, VerifyMfaRequest,
    },
    http::api::routes::api_validation::ValidatedJson,
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};
use axum::Extension;
use std::sync::Arc;

/// Start TOTP enrollment. Returns an otpauth:// URI to scan with an authenticator app.
#[utoipa::path(
    post,
    path = "/v1/api/auth/mfa/totp/start",
    responses(
        (status = 200, description = "Enrollment started — scan the otpauth URI", body = StartMfaEnrollmentResponse),
        (status = 401, description = "Missing or invalid Bearer token"),
    ),
    security(("BearerAuth" = [])),
    tag = "auth"
)]
pub async fn mfa_totp_start(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<StartMfaEnrollmentResponse>, ApiError> {
    state
        .mfa_application
        .start_enrollment(ctx.user_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// Confirm TOTP enrollment by providing the first 6-digit code from the authenticator app.
/// Returns one-time recovery codes — store them securely.
#[utoipa::path(
    post,
    path = "/v1/api/auth/mfa/totp/confirm",
    request_body = ConfirmMfaEnrollmentRequest,
    responses(
        (status = 200, description = "Enrollment confirmed — recovery codes returned", body = ConfirmMfaEnrollmentResponse),
        (status = 400, description = "MFA enrollment not started"),
        (status = 401, description = "Invalid TOTP code or missing Bearer token"),
    ),
    security(("BearerAuth" = [])),
    tag = "auth"
)]
pub async fn mfa_totp_confirm(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    ValidatedJson(request): ValidatedJson<ConfirmMfaEnrollmentRequest>,
) -> Result<RawResponse<ConfirmMfaEnrollmentResponse>, ApiError> {
    state
        .mfa_application
        .confirm_enrollment(ctx.user_id, request.code)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

/// Complete MFA login using a challenge token (from POST /login) and a TOTP or recovery code.
/// On success returns a full session token pair.
#[utoipa::path(
    post,
    path = "/v1/api/auth/mfa/verify",
    request_body = VerifyMfaRequest,
    responses(
        (status = 200, description = "MFA verified — session tokens returned", body = BasicLoginResponse),
        (status = 400, description = "MFA not enrolled"),
        (status = 401, description = "Invalid or expired MFA session token / TOTP code"),
    ),
    tag = "auth"
)]
pub async fn mfa_verify(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(request): ValidatedJson<VerifyMfaRequest>,
) -> Result<RawResponse<BasicLoginResponse>, ApiError> {
    state
        .mfa_application
        .verify_mfa_login(request.mfa_session_token, request.code)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}
