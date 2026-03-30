//! HTTP server bootstrap — [`AppState`] definition and [`HttpServer`] runner.
//!
//! [`AppState`] holds `Arc<*Application>` handles for all application-layer structs.
//! It is inserted into the Axum router via `Extension(Arc<AppState>)` so every
//! handler can access it without explicit parameter threading.
//!
//! [`HttpServer`] reads `APP_HOST` / `APP_PORT` from the environment and binds
//! a TCP listener. It supports graceful shutdown on `SIGINT` with a 1-second
//! drain period for in-flight requests.

use axum::Router;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tower::make::Shared;

use crate::{
    AdminApplication, AdminSessionApplication, ApiTokenApplication, AuthApplication,
    EmailTemplateApplication, InsightsApplication, MfaApplication, OidcApplication,
    PolicyApplication, SystemApplication, UserApplication, UserPasswordApplication,
};

/// Shared application state threaded through every Axum handler via
/// `Extension(Arc<AppState>)`.
///
/// Each field is an `Arc<ApplicationType>` — all applications are constructed
/// once in [`build_app`](crate::http::app::build_app) and shared across
/// concurrent requests without cloning.
pub struct AppState {
    pub auth_application: Arc<AuthApplication>,
    pub system_application: Arc<SystemApplication>,
    /// Holds the in-memory policy engine cache for zero-DB-call authorization checks.
    pub policy_application: Arc<PolicyApplication>,
    pub user_application: Arc<UserApplication>,
    pub oidc_application: Arc<OidcApplication>,
    /// Holds the per-token MFA attempt counter for brute-force protection.
    pub mfa_application: Arc<MfaApplication>,
    pub api_token_application: Arc<ApiTokenApplication>,
    pub user_password_application: Arc<UserPasswordApplication>,
    pub email_template_application: Arc<EmailTemplateApplication>,
    pub admin_application: Arc<AdminApplication>,
    pub admin_session_application: Arc<AdminSessionApplication>,
    pub insights_application: Arc<InsightsApplication>,
}

/// Thin wrapper that binds an Axum [`Router`] to a TCP listener and runs it.
///
/// Reads `APP_HOST` (default `0.0.0.0`) and `APP_PORT` (default `8000`) from
/// the environment. Supports graceful shutdown via `SIGINT` (Ctrl-C).
pub struct HttpServer {
    router: Router,
}

impl HttpServer {
    pub fn new(router: Router) -> Self {
        Self { router }
    }

    /// Bind to `APP_HOST:APP_PORT` and start serving requests.
    /// Blocks until a Ctrl-C signal is received, then waits 1 second for
    /// in-flight requests to complete before returning.
    pub async fn start(&self) -> anyhow::Result<()> {
        dotenv().ok(); // load .env

        let addr = env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("APP_PORT").unwrap_or_else(|_| "8000".to_string());
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", addr, port)).await?;
        tracing::info!("Server running on http://{}:{}", addr, port);

        // Start the server
        let _ = axum::serve(listener, Shared::new(self.router.clone()))
            .with_graceful_shutdown(async {
                // Wait for shutdown signal
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for shutdown signal");
                tracing::info!("Shutting down gracefully...");
                tokio::time::sleep(Duration::from_secs(1)).await; // Wait for in-flight requests
            })
            .await?;
        Ok(())
    }
}
