//! OAuth 2.0 / OIDC route definitions.
//!
//! These routes are mounted **outside** the main API router so they bypass the
//! `ResponseWrapperLayer` and return spec-compliant JSON responses directly:
//!
//! | Method | Path | Auth |
//! |--------|------|------|
//! | GET | `/.well-known/openid-configuration` | None |
//! | GET | `/oauth/jwks.json` | None |
//! | GET | `/oauth/authorize` | Sentinel Bearer token (user must be logged in) |
//! | POST | `/oauth/token` | Client credentials / PKCE |

use axum::{middleware, routing::{get, post}, Router};

use crate::http::api::{
    handlers::oidc_handlers::{authorize, jwks, openid_configuration, token_exchange},
    middlewares::authenticate_middleware::authenticate_middleware,
};

/// Routes served under /oauth (not wrapped in Sentinel ResponseWrapper envelope)
pub fn build_oauth_routes() -> Router {
    // /oauth/authorize requires auth — user must have active Sentinel PASETO session
    let auth_required = Router::new()
        .route("/authorize", get(authorize))
        .layer(middleware::from_fn(authenticate_middleware));

    Router::new()
        .route("/jwks.json", get(jwks))
        .route("/token", post(token_exchange))
        .merge(auth_required)
}

/// Routes served at root level: /.well-known/openid-configuration
pub fn build_oidc_discovery_routes() -> Router {
    Router::new().route(
        "/.well-known/openid-configuration",
        get(openid_configuration),
    )
}
