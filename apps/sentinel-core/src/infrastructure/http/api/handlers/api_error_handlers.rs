//! Fallback error handlers for the Axum router.
//!
//! These functions are registered with `.fallback()` and `HandleErrorLayer` to produce
//! consistent Sentinel envelope responses for:
//! - `not_found_handler`: 404 for any path not matched by a route
//! - `handle_timeout_error`: 408 when a request exceeds the tower timeout limit
//! - `handle_panic_error`: 500 when a handler panics (via `CatchPanicLayer`)

use crate::{api::RequestId, infrastructure::http::api::ApiResponse, ApiError};
use axum::{
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    BoxError, Extension,
};

pub async fn not_found_handler(
    Extension(request_id): Extension<RequestId>,
    request: Request<axum::body::Body>,
) -> Response {
    let path = request.uri().path().to_string();

    let api_error = ApiError {
        code: "NOT_FOUND".to_string(),
        message: format!("The endpoint '{}' does not exist", path),
        details: None,
        status: StatusCode::NOT_FOUND,
    };

    let api_response = ApiResponse::error(api_error, request_id);
    (StatusCode::NOT_FOUND, api_response).into_response()
}

pub async fn handle_global_error(err: BoxError) -> Response {
    tracing::error!("Unexpected error {:#?}", err);

    let api_general_error = ApiError {
        code: "INTERNAL_SERVER_ERROR".to_string(),
        message: "OPS! Something unexpected happen".to_string(),
        details: None,
        status: StatusCode::INTERNAL_SERVER_ERROR,
    };
    let request_id = RequestId::new();

    let api_response = ApiResponse::error(api_general_error, request_id);
    (StatusCode::INTERNAL_SERVER_ERROR, api_response).into_response()
}
