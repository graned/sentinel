mod common;

use common::{
    helpers::{post_json, read_json},
    setup::{
        get_login_user_url, get_oauth_authorize_url, get_oidc_create_client_url,
        get_oidc_generate_key_url, get_register_user_url,
    },
};
use reqwest::redirect;
use serde_json::json;
use uuid::Uuid;

/// Register a user, mark email verified, and log in. Returns the PASETO Bearer token.
async fn setup_user_and_token() -> (reqwest::Client, String) {
    let client = reqwest::Client::new();
    let email = format!("pkce-test-{}@example.com", Uuid::new_v4());
    let password = "T3stP@ssw0rd#Sec";

    let res = post_json(
        &client,
        get_register_user_url(),
        json!({
            "first_name": "PKCE",
            "last_name": "Test",
            "email": email,
            "password": password
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "registration failed: {raw}");

    // Pre-verify email via DB so authorize_middleware doesn't block
    {
        use dotenvy::dotenv;
        dotenv().ok();
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let (pg, connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
            .await
            .expect("DB connection failed");
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("DB connection error: {e}");
            }
        });
        pg.execute(
            "UPDATE user_identities SET email_verified = true WHERE email = $1",
            &[&email],
        )
        .await
        .expect("DB update failed");
    }

    let res = post_json(
        &client,
        get_login_user_url(),
        json!({ "email": email, "password": password }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");
    let token = body["data"]["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("no access_token: {raw}"))
        .to_string();

    (client, token)
}

/// Create a signing key + OIDC client. Returns `(client_id, redirect_uri)`.
async fn setup_oidc_client(client: &reqwest::Client, token: &str) -> (String, String) {
    client
        .post(get_oidc_generate_key_url())
        .bearer_auth(token)
        .send()
        .await
        .expect("generate key request failed");

    let client_id = format!("pkce-client-{}", Uuid::new_v4());
    let redirect_uri = "http://localhost:3000/callback";

    let res = client
        .post(get_oidc_create_client_url())
        .bearer_auth(token)
        .json(&json!({
            "client_id": client_id,
            "name": "PKCE Test App",
            "redirect_uris": [redirect_uri],
            "allowed_scopes": ["openid", "email"],
            "is_confidential": false,
            "pkce_required": true
        }))
        .send()
        .await
        .expect("create OIDC client request failed");
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "create OIDC client failed: {raw}");

    (client_id, redirect_uri.to_string())
}

/// Authorize request helper that does NOT follow redirects.
async fn try_authorize(
    token: &str,
    client_id: &str,
    redirect_uri: &str,
    code_challenge: &str,
    code_challenge_method: &str,
) -> reqwest::Response {
    let no_redirect = reqwest::ClientBuilder::new()
        .redirect(redirect::Policy::none())
        .build()
        .unwrap();

    no_redirect
        .get(get_oauth_authorize_url())
        .bearer_auth(token)
        .query(&[
            ("response_type", "code"),
            ("client_id", client_id),
            ("redirect_uri", redirect_uri),
            ("scope", "openid email"),
            ("state", "test_state"),
            ("code_challenge", code_challenge),
            ("code_challenge_method", code_challenge_method),
        ])
        .send()
        .await
        .expect("authorize request failed")
}

// ── Tests ──────────────────────────────────────────────────────────────────────

/// code_challenge shorter than 43 characters must be rejected.
#[tokio::test]
async fn authorize_with_too_short_code_challenge_returns_error() {
    let (client, token) = setup_user_and_token().await;
    let (client_id, redirect_uri) = setup_oidc_client(&client, &token).await;

    // 42-char challenge (one short of minimum)
    let short_challenge = "a".repeat(42);
    let res = try_authorize(&token, &client_id, &redirect_uri, &short_challenge, "S256").await;

    // Should redirect to error or return non-redirect response indicating failure
    // The OIDC endpoint returns 302 to redirect_uri with error= param on validation failure,
    // OR returns 400. Either indicates the challenge was rejected.
    assert!(
        res.status().as_u16() != 302
            || res
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .map(|loc| loc.contains("error"))
                .unwrap_or(false),
        "expected error for too-short code_challenge, got status={}",
        res.status()
    );
}

/// code_challenge containing illegal characters (not base64url) must be rejected.
#[tokio::test]
async fn authorize_with_illegal_chars_in_code_challenge_returns_error() {
    let (client, token) = setup_user_and_token().await;
    let (client_id, redirect_uri) = setup_oidc_client(&client, &token).await;

    // 43 chars but with invalid character '+' (not in base64url without padding)
    let bad_challenge = format!("{}+", "a".repeat(42));
    let res = try_authorize(&token, &client_id, &redirect_uri, &bad_challenge, "S256").await;

    assert!(
        res.status().as_u16() != 302
            || res
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .map(|loc| loc.contains("error"))
                .unwrap_or(false),
        "expected error for illegal chars in code_challenge, got status={}",
        res.status()
    );
}

/// A valid 43-character base64url code_challenge should pass format validation.
#[tokio::test]
async fn authorize_with_valid_43_char_code_challenge_succeeds() {
    let (client, token) = setup_user_and_token().await;
    let (client_id, redirect_uri) = setup_oidc_client(&client, &token).await;

    // 43-char valid base64url string (alphanumeric + '-', '_', '.', '~')
    let valid_challenge = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddEEE";
    assert_eq!(valid_challenge.len(), 43);

    let res = try_authorize(&token, &client_id, &redirect_uri, valid_challenge, "S256").await;

    // Should redirect (302 or 303) to redirect_uri with code= param
    let status_u16 = res.status().as_u16();
    assert!(
        status_u16 == 302 || status_u16 == 303,
        "expected redirect, got {status_u16}: {:?}",
        res.headers()
    );
    let location = res
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        location.contains("code="),
        "expected code in redirect, got location={location}"
    );
}
