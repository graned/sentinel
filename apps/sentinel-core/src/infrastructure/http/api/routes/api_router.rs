//! Builds the main `/v1/api/*` Axum router with all middleware and route definitions.
//!
//! All routes in this file are wrapped in the standard Sentinel JSON envelope
//! (`ResponseWrapperLayer`).  OIDC / OAuth routes are mounted **separately** in
//! `oauth_router.rs` to bypass the envelope.
//!
//! # Middleware stack (outermost → innermost)
//!
//! | Layer | Purpose |
//! |-------|---------|
//! | `CorsLayer` | Allow cross-origin requests from `CORS_ALLOWED_ORIGINS` |
//! | `RequestIdLayer` | Generate / propagate `X-Request-Id` header |
//! | `SetResponseHeaderLayer` (×3) | Inject security headers |
//! | `CatchPanicLayer` | Catch handler panics → 500 instead of process crash |
//! | `TraceLayer` | Log request + response with `tracing` |
//! | `Extension(Arc<AppState>)` | Share application state across handlers |
//! | `ResponseWrapperLayer` | Wrap all responses in `{ success, data, error, … }` |
//! | `authenticate_middleware` (selective) | Validate Bearer / API token on protected routes |
//! | `authorize_middleware` (selective) | Policy-engine authz check |

use axum::{
    extract::MatchedPath,
    http::{header, HeaderValue, Method},
    middleware,
    routing::{delete, get, post, put},
    Extension, Router,
};
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use tower_http::catch_panic::CatchPanicLayer;

use crate::{
    http::api::handlers::admin_oidc_handlers::*,
    http::api::handlers::admin_role_handlers::*,
    http::api::handlers::admin_session_handlers::*,
    http::api::handlers::admin_user_handlers::*,
    http::api::handlers::api_error_handlers::not_found_handler,
    http::api::handlers::api_token_handlers::*,
    http::api::handlers::auth_handlers::*,
    http::api::handlers::email_template_handlers::*,
    http::api::handlers::mfa_handlers::*,
    http::api::handlers::password_handlers::*,
    http::api::handlers::system_handlers::*,
    http::api::handlers::user_handlers::*,
    http::api::middlewares::{
        authenticate_middleware::authenticate_middleware,
        authorize_middleware::authorize_middleware,
        rate_limit_middleware::{moderate_limiter, rate_limit_middleware, strict_limiter},
        request_id_middleware::request_id_middleware,
        response_wrapper::ResponseWrapperLayer,
    },
    http::api::openapi::ApiDoc,
    http::api::routes::oauth_router::{build_oauth_routes, build_oidc_discovery_routes},
    http::api::RequestId,
    http::server::AppState,
};

/// Assemble the complete Axum [`Router`] for the application.
///
/// # Layer order (outermost applied last)
///
/// ```text
/// cors                    — outermost: handles OPTIONS preflight and adds CORS headers
/// request_id_middleware   — injects a unique RequestId into extensions before logging
/// Security headers        — X-Content-Type-Options, X-Frame-Options, Referrer-Policy
/// CatchPanicLayer         — converts handler panics to 500 responses
/// TraceLayer              — structured request/response logging (tower-http)
/// Extension(app_state)    — makes AppState available to all handlers via Extension extractor
/// ResponseWrapperLayer    — wraps /v1/api/* responses in the standard JSON envelope
///                           (applied only to api_router, not to OIDC/Swagger routes)
/// ```
///
/// # Route groupings
///
/// | Path prefix         | Router function        | Auth |
/// |---------------------|------------------------|------|
/// | `/v1/api/auth`      | `build_auth_routes`    | mixed (see below) |
/// | `/v1/api/user`      | `build_user_routes`    | Bearer required |
/// | `/v1/api/admin`     | `build_admin_routes`   | Bearer required (admin role enforced in app layer) |
/// | `/v1/api/system`    | `build_system_routes`  | mixed |
/// | `/oauth/*`          | OIDC routes            | none (spec-compliant) |
/// | `/.well-known/*`    | OIDC discovery         | none |
/// | `/swagger-ui`       | Swagger UI             | none |
pub fn build_router(app_state: std::sync::Arc<AppState>) -> Router {
    // CORS — read allowed origins from env; default to no origins (secure)
    let allowed_origins: Vec<HeaderValue> = std::env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    if allowed_origins.is_empty() {
        tracing::warn!("CORS_ALLOWED_ORIGINS not set — no cross-origin requests will be allowed");
    }

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(true);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            let request_id = request
                .extensions()
                .get::<RequestId>()
                .map(|r| r.as_str())
                .unwrap_or("<missing>");

            let route = request
                .extensions()
                .get::<MatchedPath>()
                .map(|p| p.as_str())
                .unwrap_or("<unmatched>");

            tracing::info_span!(
                "http_request",
                request_id = %request_id,
                route = %route,
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
            )
        })
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Seconds)
                .include_headers(true),
        );

    // API routes — all wrapped in the JSON response envelope
    let api_router = Router::new()
        .nest("/v1/api/auth", build_auth_routes())
        .nest("/v1/api/user", build_user_routes())
        .nest("/v1/api/admin", build_admin_routes())
        .nest("/v1/api/system", build_system_routes())
        .fallback(not_found_handler)
        .layer(ResponseWrapperLayer);

    Router::new()
        // Swagger UI is merged before the api_router so it is NOT wrapped by ResponseWrapperLayer
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // OIDC discovery and OAuth routes — NOT wrapped by ResponseWrapperLayer
        .merge(build_oidc_discovery_routes())
        .nest("/oauth", build_oauth_routes())
        .merge(api_router)
        .layer(Extension(app_state))
        .layer(trace_layer)
        .layer(CatchPanicLayer::new())
        // Security response headers
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(cors) // outermost (important)
        .layer(middleware::from_fn(request_id_middleware))
}

fn build_auth_routes() -> Router {
    let auth_protected = Router::new()
        .route("/logout", post(logout))
        .route("/logout-all", post(logout_all)) // requires auth
        .route("/mfa/totp/start", post(mfa_totp_start)) // requires auth
        .route("/mfa/totp/confirm", post(mfa_totp_confirm)) // requires auth
        .layer(middleware::from_fn(authenticate_middleware));

    let api_token_routes = Router::new()
        .route("/api-tokens", post(create_api_token))
        .route("/api-tokens", get(list_api_tokens))
        .route("/api-tokens/{token_id}", delete(revoke_api_token))
        .route("/api-tokens", delete(revoke_all_tokens))
        .layer(middleware::from_fn(authenticate_middleware));

    // Strict rate limit (5 req / 15 min): login and MFA verify
    let strict_limited = Router::new()
        .route("/login", post(basic_auth_login))
        .route("/mfa/verify", post(mfa_verify))
        .layer(middleware::from_fn_with_state(
            strict_limiter(),
            rate_limit_middleware,
        ));

    // Moderate rate limit (10 req / 15 min): registration, password reset, resend verification
    let moderate_limited = Router::new()
        .route("/register", post(register_user))
        .route("/password/forgot", post(forgot_password))
        .route("/resend-verification", post(resend_verification))
        .layer(middleware::from_fn_with_state(
            moderate_limiter(),
            rate_limit_middleware,
        ));

    Router::new()
        // Rate-limited sensitive routes
        .merge(strict_limited)
        .merge(moderate_limited)
        // Remaining public routes
        .route("/verify-email", get(verify_email))
        .route("/password/reset", post(reset_password))
        // Token verification
        .route("/authenticate", post(authenticate))
        // Functions that allow client to verify auth
        .route("/token/authenticate", post(authenticate))
        .route("/token/authorize", post(check_authorization))
        .route("/token/authorize/batch", post(check_authorization_batch))
        // Refresh tokens
        .route("/token/refresh", post(token_refresh))
        .route("/auth-methods", get(get_auth_methods))
        .merge(auth_protected)
        .merge(api_token_routes)
}

fn build_user_routes() -> Router {
    let auth_only = Router::new()
        .route("/me", get(get_me).patch(update_me))
        .route("/password/change", post(change_password))
        .route("/sessions", get(get_user_sessions))
        .route("/sessions/{session_id}", get(get_user_session))
        .route("/permissions", get(get_user_permissions))
        .layer(middleware::from_fn(authenticate_middleware));

    // authenticate_middleware runs first (outermost), authorize_middleware second
    let auth_and_authz = Router::new()
        .route("/canary", get(protected_canary))
        .layer(middleware::from_fn(authorize_middleware))
        .layer(middleware::from_fn(authenticate_middleware));

    auth_only.merge(auth_and_authz)
}

fn build_admin_routes() -> Router {
    Router::new()
        // Role management
        .route("/roles", post(create_role).get(list_roles))
        .route("/roles/{role_id}", put(update_role).delete(delete_role))
        // User Role Management
        // User listing and management
        .route("/users", get(list_admin_users).post(create_admin_user))
        .route("/users/{user_id}", delete(delete_admin_user))
        .route("/users/{user_id}/status", put(update_admin_user_status))
        .route("/users/{user_id}/send-invite", post(send_user_invite))
        .route("/users/{user_id}/invite-link", get(get_user_invite_link))
        // User Role Management
        .route("/users/{user_id}/roles", post(assign_role_to_user))
        .route(
            "/users/{user_id}/roles/{role_name}",
            delete(remove_role_from_user),
        )
        .route(
            "/users/{user_id}/permissions",
            get(get_user_permissions_admin),
        )
        .route("/users/{user_id}/auth-info", get(get_user_auth_info))
        .route("/users/{user_id}/mfa", put(set_user_mfa_required))
        // Policy Management
        .route("/policies", get(list_policies).post(create_policy))
        .route("/policies/{policy_id}", delete(delete_policy))
        .route(
            "/policies/{policy_id}/rules",
            get(get_policy_rules).put(update_policy_rules),
        )
        .route("/policies/{policy_id}/probe", post(run_policy_probe))
        // OIDC Management
        .route("/oidc/clients", post(create_oidc_client))
        .route("/oidc/keys/generate", post(generate_signing_key))
        // Email Templates
        .route(
            "/email-templates",
            get(list_email_templates).post(create_email_template),
        )
        .route("/email-templates/{template_id}", put(update_email_template))
        // Session management (admin)
        .route("/sessions", get(get_all_admin_sessions))
        .route("/sessions/{session_id}", delete(revoke_admin_session))
        .route("/sessions/revoke", post(revoke_admin_sessions_bulk))
        .layer(middleware::from_fn(authenticate_middleware))
}

fn build_system_routes() -> Router {
    // Protected routes (bearer token required)
    let token_protected = Router::new()
        // Provider config management
        .route(
            "/config/email",
            post(add_provider_config).get(list_provider_configs),
        )
        .route(
            "/config/email/{config_id}",
            put(update_provider_config).delete(delete_provider_config),
        )
        .route(
            "/config/email/{config_id}/reveal",
            get(get_provider_config_decrypted),
        )
        .route("/config/email/{config_id}/test", post(test_provider_config))
        .route(
            "/config/email/{config_id}/send-test",
            post(send_test_provider_email),
        )
        .route("/audit-logs", get(health_check)) // stub — requires audit_log table
        .route("/stats", get(get_insights_summary))
        .route("/analytics/user-growth", get(get_user_growth))
        .route("/analytics/sessions", get(get_session_activity))
        .route("/analytics/services", get(health_check)) // stub
        .route("/analytics/verifications", get(health_check)) // stub
        .route("/analytics/health-detailed", get(health_check)) // stub
        .layer(middleware::from_fn(authenticate_middleware));
    // API key required
    let api_token_protected = Router::new()
        // Analytics & Monitoring
        .route("/metrics", get(health_check));

    let unprotected = Router::new().route("/health", get(health_check));

    unprotected
        .merge(api_token_protected)
        .merge(token_protected)
}
