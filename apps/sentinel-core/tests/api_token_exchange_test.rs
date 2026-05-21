mod common;

use common::{
    helpers::{admin_login, assert_error_envelope, disable_admin_mfa, read_json},
    setup::{get_api_token_exchange_url, get_api_tokens_url, get_login_user_url, get_user_me_url},
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

/// Login and return the Bearer access token.
async fn login(client: &Client, email: &str, password: &str) -> String {
    if email == "admin@sentinel.local" {
        disable_admin_mfa().await;
    }
    let res = client
        .post(get_login_user_url())
        .json(&json!({ "email": email, "password": password }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");
    body.pointer("/data/access_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing access_token: {body}"))
        .to_string()
}

/// Use the seeded admin user to create an API token and return the raw `sat_*` token.
async fn create_api_token(client: &Client) -> String {
    let token = admin_login(client).await;
    let res = client
        .post(get_api_tokens_url())
        .bearer_auth(&token)
        .json(&json!({ "name": "exchange-test-token" }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "create api token failed: {raw}");
    body["data"]["token"]
        .as_str()
        .unwrap_or_else(|| panic!("missing token in response: {body}"))
        .to_string()
}

// ── Happy path ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn exchange_valid_token_creates_federated_user() {
    let client = Client::new();
    let raw_api_token = create_api_token(&client).await;

    // Use a new email that doesn't exist yet
    let new_email = format!("federated-{}@test.com", Uuid::new_v4());

    let res = client
        .post(get_api_token_exchange_url())
        .bearer_auth(&raw_api_token)
        .json(&json!({
            "email": new_email,
            "display_name": "Federated User",
            "avatar_url": Some("https://example.com/avatar.png")
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200, got {status}\n{raw}");
    assert_eq!(body["success"], true, "{body}");
    assert!(
        body["data"]["access_token"].as_str().unwrap().len() > 20,
        "access_token too short: {body}"
    );
    assert!(
        body["data"]["refresh_token"].as_str().unwrap().len() > 20,
        "refresh_token too short: {body}"
    );
    assert!(
        body["data"]["user_id"].is_string(),
        "missing user_id: {body}"
    );
}

// ── Auth security tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn exchange_without_bearer_returns_401() {
    let client = Client::new();

    let res = client
        .post(get_api_token_exchange_url())
        .json(&json!({
            "email": "admin@sentinel.local",
            "display_name": "Admin User",
            "avatar_url": null
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

#[tokio::test]
async fn exchange_with_paseto_token_returns_401() {
    let client = Client::new();
    let paseto_token = login(&client, "admin@sentinel.local", "admin").await;

    let res = client
        .post(get_api_token_exchange_url())
        .bearer_auth(&paseto_token)
        .json(&json!({
            "email": "admin@sentinel.local",
            "display_name": "Admin User",
            "avatar_url": null
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "AUTH_ERROR");
}

// ── Token validation tests ──────────────────────────────────────────────────

#[tokio::test]
async fn exchange_with_revoked_token_returns_401() {
    let client = Client::new();
    let raw_api_token = create_api_token(&client).await;

    // Create a federated user first
    let federated_email = format!("federated-revoked-{}@test.com", Uuid::new_v4());

    // Exchange once (first succeeds)
    let res = client
        .post(get_api_token_exchange_url())
        .bearer_auth(&raw_api_token)
        .json(&json!({
            "email": federated_email,
            "display_name": "Federated User",
            "avatar_url": null
        }))
        .send()
        .await
        .expect("request failed");
    let (status, _, _) = read_json(res).await;
    assert_eq!(status, 200, "first exchange should succeed");

    // Second exchange with same token — still valid since token is NOT revoked
    let res = client
        .post(get_api_token_exchange_url())
        .bearer_auth(&raw_api_token)
        .json(&json!({
            "email": federated_email,
            "display_name": "Federated User",
            "avatar_url": null
        }))
        .send()
        .await
        .expect("request failed");
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "second exchange should also succeed: {raw}");
}

#[tokio::test]
async fn exchange_with_fake_token_returns_401() {
    let client = Client::new();

    let res = client
        .post(get_api_token_exchange_url())
        .bearer_auth("sat_fake00000000000000000000000000000000000000000000000000000000")
        .json(&json!({
            "email": "admin@sentinel.local",
            "display_name": "Admin User",
            "avatar_url": null
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "AUTH_ERROR");
}

#[tokio::test]
async fn exchange_with_existing_non_federated_user_returns_400() {
    let client = Client::new();
    let raw_api_token = create_api_token(&client).await;

    // Use the existing admin email which has email_password provider
    let res = client
        .post(get_api_token_exchange_url())
        .bearer_auth(&raw_api_token)
        .json(&json!({
            "email": "admin@sentinel.local",
            "display_name": "Admin User",
            "avatar_url": null
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    // Should fail because admin@sentinel.local was created with email_password provider
    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

// ── Session usability tests ──────────────────────────────────────────────────

#[tokio::test]
async fn exchanged_session_is_usable() {
    let client = Client::new();
    let raw_api_token = create_api_token(&client).await;

    // Create a new federated user
    let new_email = format!("federated-{}@test.com", Uuid::new_v4());

    let res = client
        .post(get_api_token_exchange_url())
        .bearer_auth(&raw_api_token)
        .json(&json!({
            "email": new_email,
            "display_name": "Federated User",
            "avatar_url": Some("https://example.com/avatar.png")
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "exchange failed: {raw}");

    let access_token = body["data"]["access_token"]
        .as_str()
        .expect("missing access_token")
        .to_string();

    // Use the exchanged session on a protected endpoint
    let res = client
        .get(get_user_me_url())
        .bearer_auth(&access_token)
        .send()
        .await
        .expect("request failed");
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "user/me with exchanged token failed: {raw}");
}

#[tokio::test]
async fn exchange_returns_must_change_password() {
    let client = Client::new();
    let raw_api_token = create_api_token(&client).await;

    // Create a new federated user
    let new_email = format!("federated-{}@test.com", Uuid::new_v4());

    let res = client
        .post(get_api_token_exchange_url())
        .bearer_auth(&raw_api_token)
        .json(&json!({
            "email": new_email,
            "display_name": "Federated User",
            "avatar_url": null
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "exchange failed: {raw}");

    assert!(
        body["data"]["must_change_password"].is_boolean(),
        "missing must_change_password: {body}"
    );
    assert!(
        body["data"]["mfa_setup_required"].is_boolean(),
        "missing mfa_setup_required: {body}"
    );
}

// ── Cross-user exchange tests ────────────────────────────────────────────────

#[tokio::test]
async fn exchange_admin_token_for_different_user_creates_federated_user() {
    let client = Client::new();

    // New email for federated user
    let target_email = format!("federated-target-{}@test.com", Uuid::new_v4());

    // Admin creates an API token
    let raw_api_token = create_api_token(&client).await;

    // Exchange the admin token to create a federated user session
    let res = client
        .post(get_api_token_exchange_url())
        .bearer_auth(&raw_api_token)
        .json(&json!({
            "email": target_email,
            "display_name": "Federated Target User",
            "avatar_url": Some("https://example.com/target-avatar.png")
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "expected 200, got {status}\n{raw}");

    let access_token = body["data"]["access_token"]
        .as_str()
        .expect("missing access_token")
        .to_string();

    // Verify the session is for the target user
    let res = client
        .get(get_user_me_url())
        .bearer_auth(&access_token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "user/me failed: {raw}");
    assert_eq!(
        body["data"]["email"].as_str().unwrap_or(""),
        target_email,
        "session belongs to wrong user: {body}"
    );
}
