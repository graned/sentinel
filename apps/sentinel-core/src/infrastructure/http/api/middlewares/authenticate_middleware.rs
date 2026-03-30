//! Authentication middleware — validates Bearer tokens on every protected route.
//!
//! # Token dispatch
//!
//! The middleware detects which token type was submitted based on the prefix:
//!
//! | Token prefix | Path |
//! |-------------|------|
//! | `sat_*` | API token path — validated against `api_tokens` table |
//! | anything else | PASETO session token — decrypted and parsed in-memory |
//!
//! On success, an [`AuthenticatedUserContext`] is inserted into Axum request
//! extensions so that handlers and downstream middleware (`authorize_middleware`)
//! can read the caller's identity without re-parsing the token.
//!
//! # Must-change-password gate
//!
//! Session tokens with `mcp: true` are blocked for all endpoints except
//! `POST /v1/api/user/password/change`. API token auth bypasses this gate
//! (API tokens are not subject to password policies).

use crate::{
    api::dtos::{AuthenticateRequest, AuthenticatedUserContext},
    api::routes::api_validation::ValidatedBearer,
    server::AppState,
    ApiError, ServiceError,
};
use axum::{
    body::Body, extract::FromRequestParts, extract::OriginalUri, http::Request, middleware::Next,
    response::Response, Extension,
};
use std::sync::Arc;

/// Prefix shared by all long-lived API tokens. Used to distinguish them from PASETO session tokens.
const API_TOKEN_PREFIX: &str = "sat_";

pub async fn authenticate_middleware(
    Extension(state): Extension<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let (mut parts, body) = req.into_parts();

    // Extract Bearer Header
    let ValidatedBearer(raw_token) = ValidatedBearer::from_request_parts(&mut parts, &state)
        .await
        .map_err(|_| ServiceError::MissingTokenError("Missing auth token".to_string()))?;

    let ctx = if raw_token.starts_with(API_TOKEN_PREFIX) {
        // ── API token path ────────────────────────────────────────────────
        let auth_res = state
            .auth_application
            .authenticate_api_token(&raw_token)
            .await?;

        AuthenticatedUserContext {
            user_id: auth_res.user_id,
            session_id: auth_res.session_id,
            roles: auth_res.roles,
            bypass_authorization: true,
            email_verified: true,        // API tokens are trusted; no email gate
            must_change_password: false, // API tokens skip forced-reset gate
        }
    } else {
        // ── PASETO session token path (existing behaviour) ────────────────
        let request = AuthenticateRequest {
            access_token: raw_token,
        };
        let auth_res = state.auth_application.authenticate_token(request).await?;

        tracing::debug!("Valid request, auth context for request {:#?}", auth_res);
        AuthenticatedUserContext {
            user_id: auth_res.user_id,
            session_id: auth_res.session_id,
            roles: auth_res.roles,
            bypass_authorization: false,
            email_verified: auth_res.email_verified,
            must_change_password: auth_res.must_change_password,
        }
    };

    // Block users who must change their temporary password.
    // API token auth and the change-password endpoint itself are exempt.
    if !ctx.bypass_authorization && ctx.must_change_password {
        let path = parts
            .extensions
            .get::<OriginalUri>()
            .map(|uri| uri.path().to_string())
            .unwrap_or_else(|| parts.uri.path().to_string());

        if path != "/v1/api/user/password/change" {
            tracing::debug!(
                user_id = %ctx.user_id,
                path = %path,
                "Must change password — denying request in authenticate middleware"
            );
            return Err(ApiError::from(ServiceError::MustChangePassword(
                "You must change your password before accessing this resource".to_string(),
            )));
        }
    }

    parts.extensions.insert(ctx);

    // continue to handler
    let req = Request::from_parts(parts, body);
    Ok(next.run(req).await)
}
