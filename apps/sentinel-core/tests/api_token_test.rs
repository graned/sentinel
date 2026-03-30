mod common;

use common::{
    helpers::{admin_login, assert_error_envelope, post_json, read_json},
    setup::{
        get_api_token_url, get_api_tokens_url, get_login_user_url, get_register_user_url,
        get_user_me_url,
    },
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

// ── Setup helpers ─────────────────────────────────────────────────────────────

/// Register a fresh user and return `(email, password)`.
async fn register_user(client: &Client) -> (String, String) {
    let email = format!("api-token-user-{}@example.com", Uuid::new_v4());
    let password = "T3stP@ssw0rd#Sec";

    let res = post_json(
        client,
        get_register_user_url(),
        json!({
            "first_name": "Token",
            "last_name":  "Tester",
            "email":      email,
            "password":   password
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "register failed: {raw}");

    (email, password.to_string())
}

/// Login and return the Bearer access token.
async fn login(client: &Client, email: &str, password: &str) -> String {
    let res = post_json(
        client,
        get_login_user_url(),
        json!({ "email": email, "password": password }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");

    body.pointer("/data/access_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing access_token: {body}"))
        .to_string()
}

// ── Security tests (no admin user needed) ────────────────────────────────────

/// POST /v1/api/auth/api-tokens without a Bearer token → 401.
#[tokio::test]
async fn create_api_token_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .post(get_api_tokens_url())
        .json(&json!({ "name": "ci-bot" }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// GET /v1/api/auth/api-tokens without a Bearer token → 401.
#[tokio::test]
async fn list_api_tokens_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .get(get_api_tokens_url())
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// DELETE /v1/api/auth/api-tokens/{id} without a Bearer token → 401.
#[tokio::test]
async fn revoke_api_token_without_auth_returns_401() {
    let client = Client::new();

    let fake_id = Uuid::new_v4();
    let res = client
        .delete(get_api_token_url(fake_id))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// DELETE /v1/api/auth/api-tokens without a Bearer token → 401.
#[tokio::test]
async fn revoke_all_tokens_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .delete(get_api_tokens_url())
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// A regular user (role = "user") cannot create API tokens → 403.
#[tokio::test]
async fn create_api_token_with_non_admin_user_returns_403() {
    let client = Client::new();

    let (email, password) = register_user(&client).await;
    let token = login(&client, &email, &password).await;

    let res = client
        .post(get_api_tokens_url())
        .bearer_auth(&token)
        .json(&json!({ "name": "ci-bot" }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

/// A regular user (role = "user") cannot list API tokens → 403.
#[tokio::test]
async fn list_api_tokens_with_non_admin_user_returns_403() {
    let client = Client::new();

    let (email, password) = register_user(&client).await;
    let token = login(&client, &email, &password).await;

    let res = client
        .get(get_api_tokens_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

/// A regular user (role = "user") cannot revoke API tokens → 403.
#[tokio::test]
async fn revoke_api_token_with_non_admin_user_returns_403() {
    let client = Client::new();

    let (email, password) = register_user(&client).await;
    let token = login(&client, &email, &password).await;

    let fake_id = Uuid::new_v4();
    let res = client
        .delete(get_api_token_url(fake_id))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

/// A regular user (role = "user") cannot revoke-all API tokens → 403.
#[tokio::test]
async fn revoke_all_tokens_with_non_admin_user_returns_403() {
    let client = Client::new();

    let (email, password) = register_user(&client).await;
    let token = login(&client, &email, &password).await;

    let res = client
        .delete(get_api_tokens_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

// ── API token auth tests ──────────────────────────────────────────────────────

/// GET /v1/api/user/me with a fake `sat_*` Bearer → 401 AUTH_ERROR.
/// Proves the sat_ branch is reached and unknown tokens are rejected.
#[tokio::test]
async fn invalid_api_token_returns_401() {
    let client = Client::new();

    let res = client
        .get(get_user_me_url())
        .bearer_auth("sat_badsecrettoken000000000000000000000000000000000000000000000000")
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "AUTH_ERROR");
}

// ── Admin happy-path tests ────────────────────────────────────────────────────

/// Create an API token as an admin user; verify the raw token is returned once.
#[tokio::test]
async fn create_api_token_as_admin_returns_raw_token() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let res = client
        .post(get_api_tokens_url())
        .bearer_auth(&token)
        .json(&json!({ "name": "ci-bot" }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200, got {status}\n{raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = &body["data"];
    assert!(data["api_token_id"].is_string(), "missing api_token_id: {body}");
    assert_eq!(data["name"].as_str(), Some("ci-bot"), "{body}");

    let raw_token = data["token"].as_str().unwrap_or_else(|| panic!("missing token: {body}"));
    assert!(raw_token.starts_with("sat_"), "token must start with sat_: {raw_token}");
    assert_eq!(raw_token.len(), 68, "sat_ prefix + 64 hex chars: {raw_token}");
}

/// Full lifecycle: create → list → revoke single → list again → revoke all → list empty.
#[tokio::test]
async fn api_token_full_lifecycle() {
    let client = Client::new();
    let token = admin_login(&client).await;

    // Create token A
    let res = client
        .post(get_api_tokens_url())
        .bearer_auth(&token)
        .json(&json!({ "name": "token-a" }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "create token-a failed: {raw}");
    let token_a_id = body["data"]["api_token_id"]
        .as_str()
        .unwrap_or_else(|| panic!("missing api_token_id: {body}"))
        .to_string();

    // Create token B
    let res = client
        .post(get_api_tokens_url())
        .bearer_auth(&token)
        .json(&json!({ "name": "token-b" }))
        .send()
        .await
        .expect("request failed");
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "create token-b failed: {raw}");

    // List → should contain at least both tokens
    let res = client
        .get(get_api_tokens_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "list failed: {raw}");
    let items = body["data"].as_array().unwrap_or_else(|| panic!("data must be array: {body}"));
    assert!(items.len() >= 2, "expected at least 2 tokens: {body}");

    // Revoke token A
    let token_a_uuid: uuid::Uuid = token_a_id.parse().expect("invalid uuid");
    let res = client
        .delete(get_api_token_url(token_a_uuid))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "revoke single failed: {raw}");

    // Revoke all remaining tokens
    let res = client
        .delete(get_api_tokens_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "revoke all failed: {raw}");

    // List → all revoked (revoked_at is set, not hard-deleted); active count = 0
    let res = client
        .get(get_api_tokens_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "list after revoke-all failed: {raw}");
    let items = body["data"].as_array().unwrap_or_else(|| panic!("data must be array: {body}"));
    let active: Vec<_> = items.iter().filter(|t| t["revoked_at"].is_null()).collect();
    assert!(active.is_empty(), "expected no active tokens after revoke-all: {body}");
}
