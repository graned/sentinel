use sentinel_core::http::app::build_app;
use sentinel_core::http::server::HttpServer;
use std::env;

/// Application entry point.
///
/// Reads all required environment variables, decodes the 32-byte encryption
/// keys from their hex representations, builds the Axum router (including the
/// full dependency-injection graph), and starts the HTTP server.
///
/// # Required environment variables
/// - `DATABASE_URL`          — PostgreSQL connection string.
/// - `HEX_KEY`               — 64-char hex string (32 bytes) used to encrypt PASETO session tokens.
/// - `CONFIG_ENCRYPTION_KEY` — 64-char hex string (32 bytes) used to encrypt SMTP configs and OIDC keys.
/// - `OIDC_ISSUER_URL`       — Base URL Sentinel advertises as the OIDC issuer (defaults to `http://localhost:8080`).
/// - `FRONTEND_URL`          — Base URL of the frontend app (used in verification/reset email links).
/// - `CORS_ALLOWED_ORIGINS`  — Comma-separated list of allowed CORS origins.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured tracing — level is controlled by RUST_LOG env var.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .pretty()
        .init();

    println!("🚀 Web API server starting...");

    let db_url = env::var("DATABASE_URL").expect("Database URL must be set");
    let hex_key = env::var("HEX_KEY").expect("Hex key is missing");
    let config_key = env::var("CONFIG_ENCRYPTION_KEY").expect("Config enc key is missing");
    let oidc_issuer_url =
        env::var("OIDC_ISSUER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Decode the 64-char hex string into 32 raw bytes for PASETO symmetric encryption.
    let hex_bytes = hex::decode(hex_key).expect("Invalid hex key");
    let session_enc_key: [u8; 32] = hex_bytes
        .try_into()
        .expect("Session enc key must be 32 bytes");

    // Decode the config encryption key (used for SMTP provider secrets and OIDC RSA keys).
    let config_bytes = hex::decode(config_key).expect("Invalid config key");
    let config_enc_key: [u8; 32] = config_bytes
        .try_into()
        .expect("Config enc key must be 32 bytes");

    // Build the full application: wires together all repositories, services,
    // applications, middleware, and routes into a single Axum Router.
    let router = build_app(&db_url, session_enc_key, config_enc_key, oidc_issuer_url).await?;
    let server = HttpServer::new(router.clone());
    server.start().await.expect("❌ Failed to start server");
    Ok(())
}
