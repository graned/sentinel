//! Shared HTTP response types and the [`RequestId`] correlation-ID wrapper.
//!
//! [`ApiResponse<T>`] is the standard Sentinel envelope that wraps every
//! `/v1/api/*` response:
//!
//! ```json
//! { "success": bool, "data": T | null, "error": ApiError | null,
//!   "timestamp": "…", "request_id": "…" }
//! ```
//!
//! [`RequestId`] serialises transparently as a plain string so it appears
//! directly in the envelope instead of as a nested object.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ApiError;

// Generic API Response Wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: DateTime<Utc>,
    pub request_id: RequestId,
}
// Success response creator
impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T, request_id: impl Into<RequestId>) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            request_id: request_id.into(),
        }
    }
}

// Error response creator
impl ApiResponse<()> {
    pub fn error(error: impl Into<ApiError>, request_id: impl Into<RequestId>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            timestamp: Utc::now(),
            request_id: request_id.into(),
        }
    }
}

// Implement IntoResponse for ApiResponse
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let json_response = Json(ApiResponse {
            success: self.success,
            data: self.data,
            error: self.error.map(|e| ApiError {
                code: e.code,
                message: e.message,
                details: e.details,
                status: e.status,
            }),
            timestamp: self.timestamp,
            request_id: self.request_id,
        });

        json_response.into_response()
    }
}

// Alternative: Handler that returns raw data and lets middleware wrap it
pub struct RawResponse<T>(pub T);

impl<T: Serialize> IntoResponse for RawResponse<T> {
    fn into_response(self) -> Response {
        // This will be caught by our middleware and wrapped
        (StatusCode::OK, Json(self.0)).into_response()
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self)).into_response()
    }
}

/**
********************************************************************************************
* Request enriching
*/
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)] // serialize as a plain string
pub struct RequestId(pub String);

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Uuid> for RequestId {
    fn from(value: Uuid) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for RequestId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for RequestId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
