//! Authorization middleware — RBAC policy enforcement via the policy engine.
//!
//! # Ordering
//!
//! This middleware runs *after* `authenticate_middleware` (which populates
//! `AuthenticatedUserContext` in request extensions).
//!
//! # Bypass for API tokens
//!
//! Requests authenticated with a `sat_*` API token have
//! `ctx.bypass_authorization = true` and skip policy evaluation entirely.
//! This lets scripts and CI/CD pipelines call any endpoint without needing
//! explicit policy rules, while still requiring valid credentials.
//!
//! # Checks performed (in order)
//!
//! 1. `bypass_authorization` → pass through immediately.
//! 2. `email_verified` → deny with 403 `EMAIL_NOT_VERIFIED`.
//! 3. `must_change_password` → deny with 403 `MUST_CHANGE_PASSWORD`
//!    (except for `POST /v1/api/user/password/change`).
//! 4. Policy engine evaluation → deny with 403 `FORBIDDEN` if no rule permits.
//!
//! The full pre-strip URI is read via `OriginalUri` because this middleware is
//! nested inside `/v1/api/user` and the nested router strips the prefix from
//! `req.uri()`. Policy rules must reference the *full* path.

use crate::{api::dtos::AuthenticatedUserContext, server::AppState, ApiError, ServiceError};
use axum::{
    body::Body, extract::OriginalUri, http::Request, middleware::Next, response::Response,
    Extension,
};
use std::sync::Arc;

pub async fn authorize_middleware(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    // API-token-authenticated requests bypass policy evaluation entirely.
    if ctx.bypass_authorization {
        tracing::debug!(user_id = %ctx.user_id, "API token auth — skipping policy check");
        return Ok(next.run(req).await);
    }

    // Block unverified users before any policy evaluation.
    if !ctx.email_verified {
        tracing::debug!(user_id = %ctx.user_id, "Email not verified — denying request");
        return Err(ApiError::from(ServiceError::EmailNotVerified(
            "Email address must be verified to access this resource".to_string(),
        )));
    }

    // Block users who must change their temporary password.
    // Only the change-password endpoint is allowed through.
    let path = req
        .extensions()
        .get::<OriginalUri>()
        .map(|uri| uri.path().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());

    if ctx.must_change_password && path != "/v1/api/user/password/change" {
        tracing::debug!(user_id = %ctx.user_id, path = %path, "Must change password — denying request");
        return Err(ApiError::from(ServiceError::MustChangePassword(
            "You must change your password before accessing this resource".to_string(),
        )));
    }

    let method = req.method().as_str().to_string();

    let (allowed, _) = state
        .policy_application
        .is_allowed(None, &method, &path, &ctx.roles)
        .await
        .map_err(ApiError::from)?;

    if !allowed {
        tracing::debug!(
            user_id = %ctx.user_id,
            method = %method,
            path = %path,
            roles = ?ctx.roles,
            "Authorization denied"
        );
        return Err(ApiError::from(ServiceError::AuthorizationError(
            "Access denied".to_string(),
        )));
    }

    Ok(next.run(req).await)
}
