//! Centralised error hierarchy for Sentinel Auth.
//!
//! Errors flow through four layers, each adding context appropriate to its level:
//!
//! ```text
//! DomainError   — pure business-rule violations (no I/O concerns)
//!     ↓  From impl
//! RepositoryError — database / serialisation failures
//!     ↓  From impl
//! ServiceError  — the boundary used by the application layer;
//!                 wraps all lower errors into a flat enum
//!     ↓  From impl
//! ApiError      — HTTP-facing error; carries status code + JSON error code string
//! ```
//!
//! The [`ValidationError`] type is used by the custom Axum extractors
//! (`ValidatedJson`, `ValidatedBearer`) before a request even reaches a handler.

use crate::api::RequestId;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use diesel_async::pooled_connection::PoolError;
use rusty_paseto::prelude::{
    GeneralPasetoError, GenericBuilderError, GenericParserError, PasetoClaimError,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

// ── Validation Errors ─────────────────────────────────────────────────────────────

/// Errors produced by the custom Axum extractors *before* a handler runs.
///
/// Used by [`ValidatedJson`] and [`ValidatedBearer`] to surface JSON
/// deserialization failures, `validator` crate rule violations, or a missing
/// `Authorization` header — all with the current request ID attached so the
/// error envelope is consistent with other API errors.
#[derive(Debug)]
pub enum ValidationError {
    /// The request body could not be deserialized as JSON.
    JsonRejection {
        request_id: RequestId,
        rejection: JsonRejection,
    },
    /// The request body deserialized successfully but failed `validator` rules.
    ValidationRejection {
        request_id: RequestId,
        errors: validator::ValidationErrors,
    },
    /// The `Authorization: Bearer <token>` header was absent.
    MissingAuthToken {
        request_id: RequestId,
        message: String,
    },
}

// ── Domain Errors ─────────────────────────────────────────────────────────────

/// Pure business-rule violations with no I/O dependency.
///
/// Domain errors are the innermost error type — they know nothing about HTTP,
/// databases, or tokens. Services convert them to [`ServiceError`] via `From`.
#[derive(Error, Debug)]
pub enum DomainError {
    /// A field value or invariant check failed (e.g. password too short).
    #[error("Validation error: {0}")]
    Validation(String),

    /// A business rule was violated (e.g. email already registered).
    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    /// A required entity could not be found.
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// A catch-all for domain errors that don't fit the above categories.
    #[error("Domain error: {0}")]
    Generic(String),
}

impl DomainError {
    /// Construct a [`DomainError::Validation`] from a `&str`.
    pub fn validation(msg: &str) -> Self {
        Self::Validation(msg.to_string())
    }

    /// Construct a [`DomainError::BusinessRule`] from a `&str`.
    pub fn business_rule(msg: &str) -> Self {
        Self::BusinessRule(msg.to_string())
    }

    /// Construct a [`DomainError::NotFound`] from a `&str`.
    pub fn not_found(entity: &str) -> Self {
        Self::NotFound(entity.to_string())
    }
}

// ── Repository Errors ─────────────────────────────────────────────────────────

/// Errors that can occur inside a repository (data-access layer).
///
/// All variants are convertible to [`ServiceError`] via `From<RepositoryError>`.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// A Diesel query failed (connection issue, constraint violation, etc.).
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    /// JSON (de)serialization of a JSONB column failed.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// A data-level validation check failed before issuing the query.
    #[error("Validation error: {0}")]
    Validation(String),

    /// The query returned zero rows when exactly one was expected.
    #[error("Entity not found")]
    NotFound,

    /// A database transaction could not be completed.
    #[error("Transaction error: {0}")]
    Transaction(String),
}

// ── Service Errors ────────────────────────────────────────────────────────────

/// The primary error type used by all services and the application layer.
///
/// Handlers convert `ServiceError` into [`ApiError`] via `From<ServiceError>`,
/// which maps each variant to an appropriate HTTP status code and error-code string.
///
/// Internal errors (database failures, pool exhaustion, token-build errors) are
/// always mapped to `500 INTERNAL_ERROR` — details are never leaked to clients.
#[derive(Debug)]
pub enum ServiceError {
    /// Email already exists or another registration-time constraint was violated.
    RegisterUserError(String),
    /// A request field failed validation (maps to 400 BAD_REQUEST).
    ValidationError(String),
    /// A PASETO token could not be built (maps to 500 INTERNAL_ERROR).
    TokenBuildError(String),
    /// Credentials were invalid or the user was not found (maps to 401 UNAUTHORIZED).
    AuthenticationError(String),
    /// The authenticated user lacks permission for the requested resource (maps to 403 FORBIDDEN).
    AuthorizationError(String),
    /// A PASETO token is syntactically invalid or cannot be decrypted (maps to 401).
    InvalidTokenError(String),
    /// A PASETO token's `exp` claim has passed (maps to 401).
    ExpiredTokenError(String),
    /// No `Authorization: Bearer` header was present (maps to 401).
    MissingTokenError(String),
    /// A requested resource could not be found (maps to 404 NOT_FOUND).
    NotFoundError(String),
    /// A Diesel query failed — details are logged but not forwarded to clients.
    DatabaseError(String),
    /// The bb8 connection pool was exhausted or unavailable.
    PoolError(String),
    /// A catch-all internal error — details are logged but not forwarded to clients.
    InternalError(String),

    // ── OIDC errors ──────────────────────────────────────────────────────────
    /// The `client_id` in an OIDC request was not found (maps to 400).
    OidcClientNotFound(String),
    /// The `redirect_uri` does not match any registered URI for the client.
    OidcInvalidRedirectUri(String),
    /// One or more requested scopes are not registered for the client.
    OidcInvalidScope(String),
    /// The authorization code was not found or belongs to a different client.
    OidcInvalidCode(String),
    /// The authorization code's TTL has expired (codes are short-lived).
    OidcCodeExpired(String),
    /// The authorization code was already redeemed.
    OidcCodeConsumed(String),
    /// The PKCE `code_verifier` does not match the stored `code_challenge`.
    OidcPkceVerificationFailed(String),
    /// No RSA signing key is active — `generate_signing_key` must be called first.
    OidcNoActiveSigningKey(String),
    /// JWT signing with the active RSA key failed.
    OidcSigningError(String),

    // ── MFA errors ───────────────────────────────────────────────────────────
    /// The submitted TOTP code or recovery code was incorrect (maps to 401).
    MfaInvalidCode(String),
    /// The user attempted to verify MFA but has not enrolled TOTP (maps to 400).
    MfaNotEnrolled(String),
    /// The per-token attempt counter exceeded 5 failures within 15 minutes (maps to 429).
    MfaAttemptLimitExceeded(String),

    // ── API token errors ──────────────────────────────────────────────────────
    /// The `sat_*` API token was not found or was revoked (maps to 404).
    ApiTokenNotFound(String),

    // ── Email verification errors ─────────────────────────────────────────────
    /// The user's email is not yet verified; they must confirm it first (maps to 403).
    EmailNotVerified(String),

    // ── Forced password change ────────────────────────────────────────────────
    /// An admin-created account with a temporary password must change it first (maps to 403).
    MustChangePassword(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::AuthenticationError(msg)
            | ServiceError::AuthorizationError(msg)
            | ServiceError::ValidationError(msg)
            | ServiceError::RegisterUserError(msg)
            | ServiceError::NotFoundError(msg)
            | ServiceError::DatabaseError(msg)
            | ServiceError::PoolError(msg)
            | ServiceError::TokenBuildError(msg)
            | ServiceError::InvalidTokenError(msg)
            | ServiceError::ExpiredTokenError(msg)
            | ServiceError::MissingTokenError(msg)
            | ServiceError::InternalError(msg)
            | ServiceError::OidcClientNotFound(msg)
            | ServiceError::OidcInvalidRedirectUri(msg)
            | ServiceError::OidcInvalidScope(msg)
            | ServiceError::OidcInvalidCode(msg)
            | ServiceError::OidcCodeExpired(msg)
            | ServiceError::OidcCodeConsumed(msg)
            | ServiceError::OidcPkceVerificationFailed(msg)
            | ServiceError::OidcNoActiveSigningKey(msg)
            | ServiceError::OidcSigningError(msg)
            | ServiceError::MfaInvalidCode(msg)
            | ServiceError::MfaNotEnrolled(msg)
            | ServiceError::MfaAttemptLimitExceeded(msg)
            | ServiceError::ApiTokenNotFound(msg)
            | ServiceError::EmailNotVerified(msg)
            | ServiceError::MustChangePassword(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<bb8::RunError<PoolError>> for ServiceError {
    fn from(err: bb8::RunError<PoolError>) -> Self {
        ServiceError::PoolError(format!("Database pool error: {}", err))
    }
}

impl From<diesel::result::Error> for ServiceError {
    fn from(err: diesel::result::Error) -> Self {
        ServiceError::DatabaseError(err.to_string())
    }
}

impl From<anyhow::Error> for ServiceError {
    fn from(err: anyhow::Error) -> Self {
        ServiceError::InternalError(err.to_string())
    }
}

impl From<RepositoryError> for ServiceError {
    fn from(err: RepositoryError) -> Self {
        ServiceError::DatabaseError(err.to_string())
    }
}

impl From<DomainError> for ServiceError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::Validation(msg) => ServiceError::ValidationError(msg),
            DomainError::NotFound(msg) => ServiceError::DatabaseError(msg),
            DomainError::BusinessRule(msg) | DomainError::Generic(msg) => {
                ServiceError::InternalError(msg)
            }
        }
    }
}

// ── API Errors ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip)]
    pub status: StatusCode,
}

impl From<ServiceError> for ApiError {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::RegisterUserError(msg) | ServiceError::ValidationError(msg) => ApiError {
                code: "VALIDATION_ERROR".to_string(),
                message: msg,
                details: None,
                status: StatusCode::BAD_REQUEST,
            },
            ServiceError::InvalidTokenError(msg) => ApiError {
                code: "INVALID_TOKEN".to_string(),
                message: msg,
                details: None,
                status: StatusCode::UNAUTHORIZED,
            },
            ServiceError::ExpiredTokenError(msg) => ApiError {
                code: "EXPIRED_TOKEN".to_string(),
                message: msg,
                details: None,
                status: StatusCode::UNAUTHORIZED,
            },
            ServiceError::MissingTokenError(msg) => ApiError {
                code: "MISSING_TOKEN".to_string(),
                message: msg,
                details: None,
                status: StatusCode::UNAUTHORIZED,
            },
            ServiceError::AuthenticationError(msg) => ApiError {
                code: "AUTH_ERROR".to_string(),
                message: msg,
                details: None,
                status: StatusCode::UNAUTHORIZED,
            },
            ServiceError::AuthorizationError(msg) => ApiError {
                code: "FORBIDDEN".to_string(),
                message: msg,
                details: None,
                status: StatusCode::FORBIDDEN,
            },
            ServiceError::NotFoundError(msg) => ApiError {
                code: "NOT_FOUND".to_string(),
                message: msg,
                details: None,
                status: StatusCode::NOT_FOUND,
            },
            ServiceError::PoolError(_)
            | ServiceError::DatabaseError(_)
            | ServiceError::TokenBuildError(_)
            | ServiceError::InternalError(_)
            | ServiceError::OidcNoActiveSigningKey(_)
            | ServiceError::OidcSigningError(_) => ApiError {
                code: "INTERNAL_ERROR".to_string(),
                message: "An internal server error occurred".to_string(),
                details: None,
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
            ServiceError::OidcClientNotFound(msg) => ApiError {
                code: "OIDC_CLIENT_NOT_FOUND".to_string(),
                message: msg,
                details: None,
                status: StatusCode::BAD_REQUEST,
            },
            ServiceError::OidcInvalidRedirectUri(msg) => ApiError {
                code: "OIDC_INVALID_REDIRECT_URI".to_string(),
                message: msg,
                details: None,
                status: StatusCode::BAD_REQUEST,
            },
            ServiceError::OidcInvalidScope(msg) => ApiError {
                code: "OIDC_INVALID_SCOPE".to_string(),
                message: msg,
                details: None,
                status: StatusCode::BAD_REQUEST,
            },
            ServiceError::OidcInvalidCode(msg)
            | ServiceError::OidcCodeExpired(msg)
            | ServiceError::OidcCodeConsumed(msg)
            | ServiceError::OidcPkceVerificationFailed(msg) => ApiError {
                code: "OIDC_INVALID_GRANT".to_string(),
                message: msg,
                details: None,
                status: StatusCode::BAD_REQUEST,
            },
            ServiceError::MfaInvalidCode(msg) => ApiError {
                code: "INVALID_MFA_CODE".to_string(),
                message: msg,
                details: None,
                status: StatusCode::UNAUTHORIZED,
            },
            ServiceError::MfaNotEnrolled(msg) => ApiError {
                code: "MFA_NOT_ENROLLED".to_string(),
                message: msg,
                details: None,
                status: StatusCode::BAD_REQUEST,
            },
            ServiceError::MfaAttemptLimitExceeded(msg) => ApiError {
                code: "MFA_ATTEMPT_LIMIT_EXCEEDED".to_string(),
                message: msg,
                details: None,
                status: StatusCode::TOO_MANY_REQUESTS,
            },
            ServiceError::ApiTokenNotFound(msg) => ApiError {
                code: "API_TOKEN_NOT_FOUND".to_string(),
                message: msg,
                details: None,
                status: StatusCode::NOT_FOUND,
            },
            ServiceError::EmailNotVerified(msg) => ApiError {
                code: "EMAIL_NOT_VERIFIED".to_string(),
                message: msg,
                details: None,
                status: StatusCode::FORBIDDEN,
            },
            ServiceError::MustChangePassword(msg) => ApiError {
                code: "MUST_CHANGE_PASSWORD".to_string(),
                message: msg,
                details: None,
                status: StatusCode::FORBIDDEN,
            },
        }
    }
}

// ── PASETO Error Conversions ──────────────────────────────────────────────────

impl From<GeneralPasetoError> for ServiceError {
    fn from(err: GeneralPasetoError) -> Self {
        tracing::error!("PASETO error: {:?}", err);
        ServiceError::InternalError("Token generation failed".to_string())
    }
}

impl From<GenericParserError> for ServiceError {
    fn from(err: GenericParserError) -> Self {
        tracing::error!("PASETO parser error: {:?}", err);
        match err {
            GenericParserError::ClaimError { source }
                if matches!(source, PasetoClaimError::Expired) =>
            {
                ServiceError::ExpiredTokenError("Token expired".into())
            }
            _ => ServiceError::InvalidTokenError("Token parsing failed".into()),
        }
    }
}

impl From<GenericBuilderError> for ServiceError {
    fn from(err: GenericBuilderError) -> Self {
        tracing::error!("PASETO builder error: {:?}", err);
        ServiceError::InternalError("Token generation failed".to_string())
    }
}

impl From<PasetoClaimError> for ServiceError {
    fn from(err: PasetoClaimError) -> Self {
        tracing::error!("PASETO claim error: {:?}", err);
        ServiceError::InternalError("Token generation failed".to_string())
    }
}

impl From<serde_json::Error> for ServiceError {
    fn from(err: serde_json::Error) -> Self {
        tracing::error!("JSON parsing error: {:?}", err);
        ServiceError::InternalError("Error while parsing JSON".to_string())
    }
}
