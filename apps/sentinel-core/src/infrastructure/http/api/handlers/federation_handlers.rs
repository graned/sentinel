//! HTTP handlers for the `/v1/api/federation/*` route group.

use crate::{
    http::api::dtos::{ExchangeSupabaseTokenRequest, FederationLoginResponse},
    http::api::routes::api_validation::ValidatedJson,
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};
use axum::Extension;
use std::sync::Arc;

/// POST /v1/api/federation/supabase/exchange
///
/// Exchange a valid Supabase JWT for a native Sentinel session.
#[utoipa::path(
    post,
    path = "/v1/api/federation/supabase/exchange",
    request_body = ExchangeSupabaseTokenRequest,
    responses(
        (status = 200, description = "Token exchanged successfully", body = FederationLoginResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid or expired Supabase token"),
        (status = 404, description = "Supabase federation not enabled"),
    ),
    tag = "federation"
)]
pub async fn exchange_supabase_token(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedJson(request): ValidatedJson<ExchangeSupabaseTokenRequest>,
) -> Result<RawResponse<FederationLoginResponse>, ApiError> {
    tracing::debug!("Supabase token exchange requested");

    if !state.federation_application.is_supabase_enabled() {
        return Err(ApiError {
            code: "FEDERATION_NOT_ENABLED".to_string(),
            message: "Supabase federation is not enabled".to_string(),
            details: None,
            status: axum::http::StatusCode::NOT_FOUND,
        });
    }

    state
        .federation_application
        .exchange_supabase_token(request.access_token)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}
