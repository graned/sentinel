//! HTTP handlers for system-level endpoints (`/v1/api/system/*`).
//!
//! Covers health checks, SMTP provider configuration, and dashboard insights.
//!
//! # SMTP provider configuration
//!
//! Admins configure one or more SMTP providers (host, port, credentials/API key).
//! Secrets are encrypted at rest with XChaCha20-Poly1305; the stored
//! `config_redacted` replaces all secret values with `"****"` and is safe to return
//! in list/get responses.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET    | `/system/health` | Liveness probe |
//! | POST   | `/system/config/email` | Add an SMTP provider |
//! | GET    | `/system/config/email` | List all providers (redacted) |
//! | PUT    | `/system/config/email/{id}` | Update a provider |
//! | DELETE | `/system/config/email/{id}` | Delete a provider |
//! | GET    | `/system/config/email/{id}/reveal` | Get decrypted provider config (admin only) |
//! | POST   | `/system/config/email/{id}/test` | Test SMTP connection |
//! | POST   | `/system/config/email/{id}/send-test` | Send a test email |
//! | GET    | `/system/insights` | Dashboard aggregate metrics |

use crate::{
    http::api::dtos::{
        AuthenticatedUserContext, CreateProviderConfigRequest, DecryptedProviderConfigResponse,
        InsightsParams, InsightsSummaryResponse, ProviderConfigResponse, SendTestEmailRequest,
        SessionActivityPoint, TestProviderConfigResponse, UpdateProviderConfigRequest,
        UserGrowthPoint,
    },
    http::api::routes::api_validation::ValidatedJson,
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};

use axum::{
    extract::{Path, Query},
    Extension,
};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/v1/api/system/config/email",
    request_body = CreateProviderConfigRequest,
    responses(
        (status = 200, description = "Provider configuration saved", body = ProviderConfigResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Admin permission required"),
    ),
    security(("BearerAuth" = [])),
    tag = "system"
)]
pub async fn add_provider_config(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    ValidatedJson(request): ValidatedJson<CreateProviderConfigRequest>,
) -> Result<RawResponse<ProviderConfigResponse>, ApiError> {
    tracing::debug!("CREATE Provider configuration request {:#?}", request);
    match state
        .system_application
        .add_provider_config(ctx, request)
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
    get,
    path = "/v1/api/system/config/email",
    responses(
        (status = 200, description = "List of provider configurations (redacted)", body = Vec<ProviderConfigResponse>),
        (status = 401, description = "Missing or invalid token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "system"
)]
pub async fn list_provider_configs(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<Vec<ProviderConfigResponse>>, ApiError> {
    state
        .system_application
        .list_configs(&ctx)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    put,
    path = "/v1/api/system/config/email/{config_id}",
    params(("config_id" = Uuid, Path, description = "Provider configuration ID")),
    request_body = UpdateProviderConfigRequest,
    responses(
        (status = 200, description = "Provider configuration updated", body = ProviderConfigResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Missing or invalid token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Configuration not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "system"
)]
pub async fn update_provider_config(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(config_id): Path<Uuid>,
    ValidatedJson(request): ValidatedJson<UpdateProviderConfigRequest>,
) -> Result<RawResponse<ProviderConfigResponse>, ApiError> {
    state
        .system_application
        .update_config(&ctx, config_id, request)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    delete,
    path = "/v1/api/system/config/email/{config_id}",
    params(("config_id" = Uuid, Path, description = "Provider configuration ID")),
    responses(
        (status = 200, description = "Provider configuration deleted"),
        (status = 401, description = "Missing or invalid token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Configuration not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "system"
)]
pub async fn delete_provider_config(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(config_id): Path<Uuid>,
) -> Result<RawResponse<()>, ApiError> {
    state
        .system_application
        .delete_config(&ctx, config_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/v1/api/system/config/email/{config_id}/reveal",
    params(("config_id" = Uuid, Path, description = "Provider configuration ID")),
    responses(
        (status = 200, description = "Decrypted provider configuration", body = DecryptedProviderConfigResponse),
        (status = 401, description = "Missing or invalid token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Configuration not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "system"
)]
pub async fn get_provider_config_decrypted(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(config_id): Path<Uuid>,
) -> Result<RawResponse<DecryptedProviderConfigResponse>, ApiError> {
    state
        .system_application
        .get_decrypted_config(&ctx, config_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/v1/api/system/config/email/{config_id}/test",
    params(("config_id" = Uuid, Path, description = "Provider configuration ID")),
    responses(
        (status = 200, description = "Connection test result", body = TestProviderConfigResponse),
        (status = 401, description = "Missing or invalid token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Configuration not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "system"
)]
pub async fn test_provider_config(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(config_id): Path<Uuid>,
) -> Result<RawResponse<TestProviderConfigResponse>, ApiError> {
    state
        .system_application
        .test_config(&ctx, config_id)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/v1/api/system/config/email/{config_id}/send-test",
    params(("config_id" = Uuid, Path, description = "Provider configuration ID")),
    request_body = SendTestEmailRequest,
    responses(
        (status = 200, description = "Send test email result", body = TestProviderConfigResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Missing or invalid token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Configuration not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "system"
)]
pub async fn send_test_provider_email(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(config_id): Path<Uuid>,
    ValidatedJson(request): ValidatedJson<SendTestEmailRequest>,
) -> Result<RawResponse<TestProviderConfigResponse>, ApiError> {
    state
        .system_application
        .send_test_email_config(&ctx, config_id, &request.to_email)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

// ─── Analytics / insights handlers ───────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/v1/api/system/stats",
    responses(
        (status = 200, description = "Platform KPI snapshot", body = InsightsSummaryResponse),
        (status = 401, description = "Missing or invalid token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "system"
)]
pub async fn get_insights_summary(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<InsightsSummaryResponse>, ApiError> {
    state
        .insights_application
        .get_summary(&ctx)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/v1/api/system/analytics/user-growth",
    params(InsightsParams),
    responses(
        (status = 200, description = "Cumulative user-growth time series", body = Vec<UserGrowthPoint>),
        (status = 401, description = "Missing or invalid token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "system"
)]
pub async fn get_user_growth(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Query(params): Query<InsightsParams>,
) -> Result<RawResponse<Vec<UserGrowthPoint>>, ApiError> {
    state
        .insights_application
        .get_user_growth(&ctx, params.days)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/v1/api/system/analytics/sessions",
    params(InsightsParams),
    responses(
        (status = 200, description = "Daily session-activity time series", body = Vec<SessionActivityPoint>),
        (status = 401, description = "Missing or invalid token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "system"
)]
pub async fn get_session_activity(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Query(params): Query<InsightsParams>,
) -> Result<RawResponse<Vec<SessionActivityPoint>>, ApiError> {
    state
        .insights_application
        .get_session_activity(&ctx, params.days)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}

pub async fn health_check() -> Result<String, ApiError> {
    Ok("Okiley Dokiley!".to_string())
}

#[utoipa::path(
    get,
    path = "/v1/api/user/canary",
    responses(
        (status = 200, description = "Authentication and authorization verified", body = String),
        (status = 401, description = "Missing or invalid token"),
        (status = 403, description = "Access denied by policy"),
    ),
    security(("BearerAuth" = [])),
    tag = "user"
)]
pub async fn protected_canary(
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<String>, ApiError> {
    let msg = format!(
        "Halt! Who goes there? 🛡️  Ah, it's '{}' — the sentinels recognise your {:?} credentials. You shall pass!",
        ctx.user_id, ctx.roles
    );
    Ok(RawResponse(msg))
}
