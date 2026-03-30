//! Custom Axum extractors with integrated validation.
//!
//! # [`ValidatedJson<T>`]
//!
//! A drop-in replacement for Axum's `Json<T>` that also runs
//! `T::validate()` (from the `validator` crate) after deserialisation.
//! On failure, it returns a structured [`ValidationError`] which the
//! `IntoResponse` impl converts to a 400 response inside the API envelope.
//!
//! ```rust,ignore
//! async fn handler(ValidatedJson(body): ValidatedJson<RegisterUserRequest>) { ... }
//! ```
//!
//! # [`ValidatedBearer`]
//!
//! Extracts the raw token string from an `Authorization: Bearer <token>` header.
//! Returns [`ValidationError::MissingAuthToken`] (→ 401) when the header is absent.
//!
//! ```rust,ignore
//! async fn handler(ValidatedBearer(token): ValidatedBearer) { ... }
//! ```

use axum::{
    http::{request::Parts, Request, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::{
    api::{ApiResponse, RequestId},
    ApiError, ValidationError,
};

/// Axum extractor that deserialises a JSON body and then runs `T::validate()`.
/// Produces a structured 400 response on any validation failure.
#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<T> std::ops::Deref for ValidatedJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for ValidatedJson<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, S> axum::extract::FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send,
    S: Send + Sync,
{
    type Rejection = ValidationError;

    async fn from_request(
        req: Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();

        let request_id = parts
            .extensions
            .get::<RequestId>()
            .cloned()
            .unwrap_or_else(RequestId::new);
        let req = Request::from_parts(parts, body);

        // Validate JSON structure
        let Json(value) = Json::<T>::from_request(req, state).await.map_err(|rej| {
            ValidationError::JsonRejection {
                request_id: request_id.clone(),
                rejection: rej,
            }
        })?;

        // Validate incoming business rules (types, length, etc)
        value
            .validate()
            .map_err(|verr| ValidationError::ValidationRejection {
                request_id: request_id.clone(),
                errors: verr,
            })?;

        Ok(ValidatedJson(value))
    }
}
/// Axum extractor that pulls the raw bearer token string out of
/// `Authorization: Bearer <token>`. Returns 401 when the header is absent.
#[derive(Debug)]
pub struct ValidatedBearer(pub String);

impl std::ops::Deref for ValidatedBearer {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> axum::extract::FromRequestParts<S> for ValidatedBearer
where
    S: Send + Sync,
{
    type Rejection = ValidationError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let request_id = parts
            .extensions
            .get::<RequestId>()
            .cloned()
            .unwrap_or_else(RequestId::new);

        // Try to extract Authorization: Bearer <token>
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| ValidationError::MissingAuthToken {
                    request_id: request_id.clone(),
                    message: "Bearer token is empty".to_string(),
                })?;

        let token = bearer.token().trim();
        Ok(ValidatedBearer(token.to_string()))
    }
}
//******************************************************************************************

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        match self {
            ValidationError::JsonRejection {
                request_id,
                rejection,
            } => {
                let (message, details) = match rejection {
                    axum::extract::rejection::JsonRejection::JsonDataError(err) => {
                        // Extract more specific error information
                        let error_string = err.to_string();
                        let field_hint = extract_missing_field(&error_string);

                        let details = serde_json::json!({
                            "parse_error": error_string,
                            "missing_field": field_hint,
                            "hint": "Check that all required fields are present and have correct types"
                        });
                        ("Invalid request data".to_string(), Some(details))
                    }
                    axum::extract::rejection::JsonRejection::JsonSyntaxError(err) => {
                        let details = serde_json::json!({
                            "syntax_error": err.to_string(),
                            "hint": "Check your JSON syntax (missing commas, quotes, etc.)"
                        });
                        ("Invalid JSON syntax".to_string(), Some(details))
                    }
                    axum::extract::rejection::JsonRejection::MissingJsonContentType(_) => (
                        "Missing Content-Type: application/json header".to_string(),
                        None,
                    ),
                    axum::extract::rejection::JsonRejection::BytesRejection(_) => {
                        ("Failed to read request body".to_string(), None)
                    }
                    _ => ("Invalid JSON request".to_string(), None),
                };

                let api_error = ApiError {
                    code: "INVALID_JSON".to_string(),
                    message,
                    details,
                    status: StatusCode::BAD_REQUEST,
                };
                let api_response = ApiResponse::error(api_error, request_id);
                (StatusCode::BAD_REQUEST, api_response).into_response()
            }
            ValidationError::ValidationRejection { request_id, errors } => {
                let details = serde_json::to_value(&errors).unwrap_or(serde_json::Value::Null);

                let api_error = ApiError {
                    code: "VALIDATION_ERROR".to_string(),
                    message: "Request validation failed".to_string(),
                    details: Some(details),
                    status: StatusCode::BAD_REQUEST,
                };
                let api_response = ApiResponse::error(api_error, request_id);
                (StatusCode::BAD_REQUEST, api_response).into_response()
            }
            ValidationError::MissingAuthToken {
                request_id,
                message,
            } => {
                let api_error = ApiError {
                    code: "MISSING_TOKEN".to_string(),
                    message,
                    details: None,
                    status: StatusCode::UNAUTHORIZED,
                };
                let api_response = ApiResponse::error(api_error, request_id);
                (StatusCode::UNAUTHORIZED, api_response).into_response()
            }
        }
    }
}

// Helper function to extract missing field from serde error
fn extract_missing_field(error_msg: &str) -> Option<String> {
    if error_msg.contains("missing field") {
        // Error format: "missing field `email` at line 1 column 100"
        if let Some(start) = error_msg.find('`') {
            if let Some(end) = error_msg[start + 1..].find('`') {
                return Some(error_msg[start + 1..start + 1 + end].to_string());
            }
        }
    }
    None
}
