//! HTTP handlers for the `/v1/api/auth/*` route group.
//!
//! All handlers delegate to `AuthApplication` — they are thin glue that:
//! 1. Extract and validate the request (via `ValidatedJson` / `ValidatedBearer` / `Query`)
//! 2. Call the corresponding application method
//! 3. Convert `ServiceError` → `ApiError` via the `From` impl
//!
//! # Endpoints in this file
//!
//! | Method | Path | Handler | Notes |
//! |--------|------|---------|-------|
//! | POST | `/auth/register` | `register_user` | Creates user + identity; sends verification email |
//! | POST | `/auth/login` | `basic_auth_login` | Returns session or MFA challenge |
//! | POST | `/auth/authenticate` | `authenticate` | Validates Bearer token; returns claims |
//! | POST | `/auth/logout` | `logout` | Revokes one session |
//! | POST | `/auth/logout-all` | `logout_all` | Revokes all sessions for the user |
//! | POST | `/auth/token/authorize` | `check_authorization` | Policy-engine authorization check |
//! | POST | `/auth/token/refresh` | `token_refresh` | Rotate refresh token → new session |
//! | GET  | `/auth/verify-email` | `verify_email` | Consume `?token=<ev_*>` |
//! | POST | `/auth/resend-verification` | `resend_verification` | Resend the email link |
//! | GET  | `/auth/auth-methods` | `get_auth_methods` | List enabled auth methods |
//! | POST | `/auth/api-tokens` | `create_api_token` | Create long-lived API token |
//! | GET  | `/auth/api-tokens` | `list_api_tokens` | List caller's API tokens |
//! | DELETE | `/auth/api-tokens/{id}` | `revoke_api_token` | Soft-revoke one token |
//! | DELETE | `/auth/api-tokens` | `revoke_all_tokens` | Soft-revoke all tokens |
//! | POST | `/admin/policies` | `create_policy` | Admin: create a policy |
//! | PUT  | `/admin/policies/{id}/rules` | `update_policy_rules` | Admin: compile + activate rules |

use crate::{
    http::api::dtos::{
        AuthContextResponse, AuthMethodsResponse, AuthenticateRequest, AuthenticatedUserContext,
        BasicAuthLoginRequest, BasicLoginResponse, BatchCheckRequest, BatchCheckResponse,
        CheckAuthorizationRequest, CheckAuthorizationResponse, CreatePolicyRequest,
        CreatePolicyResponse, GetPolicyRulesResponse, LoginOutcome, PolicyResponse,
        RefreshTokenRequest, RegisterUserRequest, RegisterUserResponse, ResendVerificationRequest,
        RunProbeRequest, RunProbeResponse, UpdatePolicyRulesRequest, UpdatePolicyRulesResponse,
        VerifyEmailQuery,
    },
    http::api::routes::api_validation::{ValidatedBearer, ValidatedJson},
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};
use axum::extract::{Path, Query};
use axum::Extension;
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/v1/api/auth/authenticate",
    responses(
        (status = 200, description = "Token is valid", body = AuthContextResponse),
        (status = 401, description = "Invalid or expired token"),
    ),
    security(("BearerAuth" = [])),
    tag = "auth"
)]
pub async fn authenticate(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedBearer(access_token): ValidatedBearer,
) -> Result<RawResponse<AuthContextResponse>, ApiError> {
    let request = AuthenticateRequest {
        access_token,
    };
    match state.auth_application.authenticate_token(request).await {
        Ok(res) => {
            tracing::debug!("Valid token for {:#?}", res);
            Ok(RawResponse(res))
        }
        Err(err) => Err(ApiError::from(err)),
    }
}

#[utoipa::path(
    post,
    path = "/v1/api/auth/logout",
    responses(
        (status = 200, description = "Logged out — session revoked"),
        (status = 401, description = "Missing or invalid token"),
    ),
    security(("BearerAuth" = [])),
    tag = "auth"
)]
pub async fn logout(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<String>, ApiError> {
    state
        .auth_application
        .logout(ctx)
        .await
        .map(|_| RawResponse("Logged out successfully".to_string()))
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/v1/api/auth/logout-all",
    responses(
        (status = 200, description = "All sessions revoked"),
        (status = 401, description = "Missing or invalid token"),
    ),
    security(("BearerAuth" = [])),
    tag = "auth"
)]
pub async fn logout_all(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<String>, ApiError> {
    state
        .auth_application
        .logout_all(ctx)
        .await
        .map(|_| RawResponse("All sessions revoked successfully".to_string()))
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/v1/api/auth/register",
    request_body = RegisterUserRequest,
    responses(
        (status = 200, description = "User registered successfully", body = RegisterUserResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already in use"),
    ),
    tag = "auth"
)]
pub async fn register_user(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(request): ValidatedJson<RegisterUserRequest>,
) -> Result<RawResponse<RegisterUserResponse>, ApiError> {
    tracing::debug!("Register user request {:#?}", request);
    match state
        .auth_application
        .register_with_basic_auth(request)
        .await
    {
        Ok(res) => {
            tracing::debug!("Parsed response {:#?}", res);
            Ok(RawResponse(res))
        }
        Err(err) => Err(ApiError::from(err)),
    }
}

#[utoipa::path(
    post,
    path = "/v1/api/auth/login",
    request_body = BasicAuthLoginRequest,
    responses(
        (status = 200, description = "Login successful — tokens or MFA challenge", body = LoginOutcome),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid credentials"),
    ),
    tag = "auth"
)]
pub async fn basic_auth_login(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(request): ValidatedJson<BasicAuthLoginRequest>,
) -> Result<RawResponse<LoginOutcome>, ApiError> {
    tracing::debug!("Basic auth login {:#?}", request);
    match state.auth_application.basic_auth_login(request).await {
        Ok(res) => {
            tracing::debug!("Basic auth login response {:#?}", res);
            Ok(RawResponse(res))
        }
        Err(err) => Err(ApiError::from(err)),
    }
}

#[utoipa::path(
    post,
    path = "/v1/api/auth/token/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "New token pair issued", body = BasicLoginResponse),
        (status = 401, description = "Refresh token invalid, expired, or revoked"),
    ),
    tag = "auth"
)]
pub async fn token_refresh(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(request): ValidatedJson<RefreshTokenRequest>,
) -> Result<RawResponse<BasicLoginResponse>, ApiError> {
    state
        .auth_application
        .refresh_token(request)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/v1/api/admin/policies",
    responses(
        (status = 200, description = "List of all policies", body = Vec<PolicyResponse>),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn list_policies(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<Vec<PolicyResponse>>, ApiError> {
    state
        .policy_application
        .list_policies(&ctx)
        .await
        .map(|policies| RawResponse(policies.into_iter().map(PolicyResponse::from).collect()))
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/v1/api/admin/policies",
    request_body = CreatePolicyRequest,
    responses(
        (status = 200, description = "Policy created", body = CreatePolicyResponse),
        (status = 400, description = "Validation error"),
    ),
    tag = "admin"
)]
pub async fn create_policy(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(request): ValidatedJson<CreatePolicyRequest>,
) -> Result<RawResponse<CreatePolicyResponse>, ApiError> {
    tracing::debug!("Create policy request: {:?}", request.name);
    match state.policy_application.create_policy(request).await {
        Ok(res) => Ok(RawResponse(res)),
        Err(err) => Err(ApiError::from(err)),
    }
}

#[utoipa::path(
    put,
    path = "/v1/api/admin/policies/{policy_id}/rules",
    params(
        ("policy_id" = Uuid, Path, description = "UUID of the policy to update"),
    ),
    request_body = UpdatePolicyRulesRequest,
    responses(
        (status = 200, description = "Policy rules compiled and activated", body = UpdatePolicyRulesResponse),
        (status = 400, description = "Validation error — invalid rules format"),
        (status = 404, description = "Policy not found"),
    ),
    tag = "admin"
)]
pub async fn update_policy_rules(
    Extension(state): Extension<Arc<AppState>>,
    Path(policy_id): Path<Uuid>,
    ValidatedJson(request): ValidatedJson<UpdatePolicyRulesRequest>,
) -> Result<RawResponse<UpdatePolicyRulesResponse>, ApiError> {
    tracing::debug!("Update policy rules for policy {}", policy_id);
    match state
        .policy_application
        .update_policy_rules(policy_id, request)
        .await
    {
        Ok(res) => Ok(RawResponse(res)),
        Err(err) => Err(ApiError::from(err)),
    }
}

#[utoipa::path(
    get,
    path = "/v1/api/admin/policies/{policy_id}/rules",
    params(
        ("policy_id" = Uuid, Path, description = "UUID of the policy"),
    ),
    responses(
        (status = 200, description = "Active rules for this policy", body = GetPolicyRulesResponse),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Policy not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn get_policy_rules(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(policy_id): Path<Uuid>,
) -> Result<RawResponse<GetPolicyRulesResponse>, ApiError> {
    state
        .policy_application
        .get_active_rules(&ctx, policy_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    delete,
    path = "/v1/api/admin/policies/{policy_id}",
    params(
        ("policy_id" = Uuid, Path, description = "UUID of the policy to delete"),
    ),
    responses(
        (status = 200, description = "Policy deleted"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Policy not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn delete_policy(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(policy_id): Path<Uuid>,
) -> Result<RawResponse<()>, ApiError> {
    state
        .policy_application
        .delete_policy(&ctx, policy_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/v1/api/auth/token/authorize",
    request_body = CheckAuthorizationRequest,
    responses(
        (status = 200, description = "Authorization check result", body = CheckAuthorizationResponse),
        (status = 400, description = "Validation error"),
    ),
    tag = "auth"
)]
pub async fn check_authorization(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(request): ValidatedJson<CheckAuthorizationRequest>,
) -> Result<RawResponse<CheckAuthorizationResponse>, ApiError> {
    tracing::debug!(
        "Check authorization — {} {} (policy: {:?})",
        request.method,
        request.path,
        request.policy_id
    );
    match state
        .policy_application
        .is_allowed(
            request.policy_id,
            &request.method,
            &request.path,
            &request.roles,
        )
        .await
    {
        Ok((allowed, active_version)) => Ok(RawResponse(CheckAuthorizationResponse {
            allowed,
            method: request.method,
            path: request.path,
            roles: request.roles,
            active_version,
        })),
        Err(err) => Err(ApiError::from(err)),
    }
}

#[utoipa::path(
    get,
    path = "/v1/api/auth/verify-email",
    params(VerifyEmailQuery),
    responses(
        (status = 200, description = "Email verified successfully"),
        (status = 401, description = "Invalid or expired token"),
    ),
    tag = "auth"
)]
pub async fn verify_email(
    Extension(state): Extension<Arc<AppState>>,
    Query(params): Query<VerifyEmailQuery>,
) -> Result<RawResponse<String>, ApiError> {
    state
        .auth_application
        .verify_email(params.token)
        .await
        .map(|_| RawResponse("Email verified successfully".to_string()))
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/v1/api/auth/resend-verification",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Verification email sent if address is registered and unverified"),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Email not found"),
    ),
    tag = "auth"
)]
pub async fn resend_verification(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<ResendVerificationRequest>,
) -> Result<RawResponse<String>, ApiError> {
    state
        .auth_application
        .resend_verification(req.email)
        .await
        .map(|_| RawResponse("Verification email sent".to_string()))
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/v1/api/auth/auth-methods",
    responses(
        (status = 200, description = "Available authentication methods", body = AuthMethodsResponse),
    ),
    tag = "auth"
)]
pub async fn get_auth_methods(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<RawResponse<AuthMethodsResponse>, ApiError> {
    state
        .system_application
        .get_auth_methods()
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/v1/api/auth/token/authorize/batch",
    request_body = BatchCheckRequest,
    responses(
        (status = 200, description = "Batch authorization check results", body = BatchCheckResponse),
        (status = 400, description = "Validation error"),
    ),
    tag = "auth"
)]
pub async fn check_authorization_batch(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(request): ValidatedJson<BatchCheckRequest>,
) -> Result<RawResponse<BatchCheckResponse>, ApiError> {
    state
        .policy_application
        .check_authorization_batch(request)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/v1/api/admin/policies/{policy_id}/probe",
    params(
        ("policy_id" = Uuid, Path, description = "UUID of the policy to probe"),
    ),
    request_body = RunProbeRequest,
    responses(
        (status = 200, description = "Probe results for each rule", body = RunProbeResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Policy not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn run_policy_probe(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(policy_id): Path<Uuid>,
    ValidatedJson(request): ValidatedJson<RunProbeRequest>,
) -> Result<RawResponse<RunProbeResponse>, ApiError> {
    state
        .policy_application
        .run_probe(&ctx, policy_id, request)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}
