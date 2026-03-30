//! DTOs for MFA (TOTP) endpoints: login branching (`LoginOutcome`), enrollment,
//! and MFA challenge verification.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::BasicLoginResponse;

// ── Login branching ────────────────────────────────────────────────────────────

/// Returned by POST /v1/api/auth/login.
/// Non-MFA users receive the `Success` variant (identical JSON to before).
/// MFA-enrolled users receive the `MfaChallenge` variant.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum LoginOutcome {
    Success(BasicLoginResponse),
    MfaChallenge(MfaChallengeResponse),
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MfaChallengeResponse {
    pub user_id: Uuid,
    pub mfa_required: bool,
    pub mfa_session_token: String,
}

// ── Enrollment ─────────────────────────────────────────────────────────────────

/// Response for POST /v1/api/auth/mfa/totp/start
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StartMfaEnrollmentResponse {
    pub otpauth_uri: String,
}

/// Request for POST /v1/api/auth/mfa/totp/confirm
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct ConfirmMfaEnrollmentRequest {
    #[validate(length(equal = 6))]
    pub code: String,
}

/// Response for POST /v1/api/auth/mfa/totp/confirm
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ConfirmMfaEnrollmentResponse {
    pub recovery_codes: Vec<String>,
}

// ── MFA login verification ─────────────────────────────────────────────────────

/// Request for POST /v1/api/auth/mfa/verify
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct VerifyMfaRequest {
    #[validate(length(min = 1))]
    pub mfa_session_token: String,
    #[validate(length(min = 1))]
    pub code: String,
}
