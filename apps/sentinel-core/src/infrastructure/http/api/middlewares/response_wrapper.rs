//! Tower [`Layer`] that wraps every `v1/api/*` response in the standard
//! Sentinel envelope:
//!
//! ```json
//! {
//!   "success": true,
//!   "data": { ... },
//!   "error": null,
//!   "timestamp": "2026-03-21T12:00:00Z",
//!   "request_id": "01HX..."
//! }
//! ```
//!
//! The wrapper is applied via [`ResponseWrapperLayer`] and skipped for
//! OIDC/OAuth routes (mounted before the layer) and the Swagger UI.
//!
//! # Already-enveloped responses
//!
//! If the inner service already returned a response with `success`, `request_id`,
//! and `timestamp` fields, the wrapper passes it through unchanged to avoid
//! double-wrapping (e.g. when a handler returns `ApiError` directly).

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use uuid::Uuid;

use crate::{
    api::{ApiResponse, RequestId},
    ApiError,
};

/// Request extension carrying a UUID for the current request.
/// Populated by `request_id_middleware` and forwarded to the response envelope.
#[derive(Clone, Default)]
pub struct RequestContext {
    pub request_id: Uuid,
}

/// Tower [`Layer`] that produces a [`ResponseWrapper`] service.
/// Apply this layer to any router whose responses should be enveloped.
#[derive(Clone)]
pub struct ResponseWrapperLayer;

impl<S> Layer<S> for ResponseWrapperLayer {
    type Service = ResponseWrapper<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ResponseWrapper { inner }
    }
}

/// Tower [`Service`] that intercepts responses from the inner service and
/// wraps them in the standard [`ApiResponse`] envelope.
#[derive(Clone)]
pub struct ResponseWrapper<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for ResponseWrapper<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let request_id = req
                .extensions()
                .get::<RequestId>()
                .cloned()
                .unwrap_or_else(RequestId::new);
            let response = inner.call(req).await?;
            Ok(wrap_response(response, request_id).await)
        })
    }
}

// Function to wrap responses
async fn wrap_response(response: Response, request_id: impl Into<RequestId>) -> Response {
    let status = response.status();
    let (parts, body) = response.into_parts();

    // Convert the body to bytes to inspect it, with a max size of 5MB
    let bytes = match axum::body::to_bytes(body, (1024 * 1024) * 5).await {
        Ok(bytes) => bytes,
        Err(_) => {
            // If we can't read the body, return a generic success
            let api_response = ApiResponse::success((), request_id);
            return api_response.into_response();
        }
    };

    // Check if body is empty
    if bytes.is_empty() {
        let api_response = ApiResponse::success((), request_id);
        return api_response.into_response();
    }
    let json_value = serde_json::from_slice::<serde_json::Value>(&bytes).ok();

    if let Some(ref v) = json_value {
        let already_enveloped = v.get("success").is_some()
            && v.get("request_id").is_some()
            && v.get("timestamp").is_some()
            && (v.get("data").is_some() || v.get("error").is_some());

        if already_enveloped {
            return Response::from_parts(parts, Body::from(bytes));
        }
    }
    // Handle responses different from 2xx
    if !status.is_success() {
        if let Some(v) = json_value.clone() {
            if let Ok(mut api_error) = serde_json::from_value::<ApiError>(v) {
                // Ensure status matches HTTP status
                api_error.status = status;
                return (status, Json(ApiResponse::error(api_error, request_id))).into_response();
            }
        }
    }
    // Try to parse as JSON
    match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(json_value) => {
            // Successfully parsed JSON - wrap it
            let api_response = ApiResponse {
                success: true,
                data: Some(json_value),
                error: None,
                timestamp: chrono::Utc::now(),
                request_id: request_id.into(),
            };
            return (StatusCode::OK, Json(api_response)).into_response();
        }
        Err(_) => {
            // Not JSON or invalid JSON - check if it's a string
            if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                // It's a string response - wrap it
                let api_response = ApiResponse {
                    success: true,
                    data: Some(serde_json::Value::String(text)),
                    error: None,
                    timestamp: chrono::Utc::now(),
                    request_id: request_id.into(),
                };
                (StatusCode::OK, Json(api_response)).into_response()
            } else {
                // Binary or other data - return original response
                Response::from_parts(parts, Body::from(bytes))
            }
        }
    }
}
