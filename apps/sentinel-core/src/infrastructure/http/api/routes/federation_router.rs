//! Federation routes — `/v1/api/federation/*`

use crate::infrastructure::http::api::handlers::federation_handlers;
use axum::{routing::post, Router};

/// Build the federation router with all federation-related endpoints.
pub fn build_federation_routes() -> Router {
    Router::new().route(
        "/supabase/exchange",
        post(federation_handlers::exchange_supabase_token),
    )
}
