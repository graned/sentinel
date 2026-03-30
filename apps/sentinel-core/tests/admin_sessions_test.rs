mod common;

use common::{
    helpers::{admin_login, assert_error_envelope, post_json, read_json, TEST_PASSWORD},
    setup::{
        get_admin_session_url, get_admin_sessions_revoke_url, get_admin_sessions_url,
        get_login_user_url, get_register_user_url, get_user_sessions_url,
    },
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

// ── Setup helpers ─────────────────────────────────────────────────────────────

/// Register a fresh user and return `(email, password)`.
async fn register_user(client: &Client) -> (String, String) {
    let email = format!("admin-session-user-{}@example.com", Uuid::new_v4());
    let res = post_json(
        client,
        get_register_user_url(),
        json!({
            "first_name": "Session",
            "last_name":  "Tester",
            "email":      email,
            "password":   TEST_PASSWORD
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "register failed: {raw}");
    (email, TEST_PASSWORD.to_string())
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

// ── Security tests — no token ─────────────────────────────────────────────────

/// GET /v1/api/admin/sessions without a Bearer token → 401.
#[tokio::test]
async fn list_admin_sessions_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .get(get_admin_sessions_url())
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// DELETE /v1/api/admin/sessions/{id} without a Bearer token → 401.
#[tokio::test]
async fn revoke_admin_session_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .delete(get_admin_session_url(Uuid::new_v4()))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// POST /v1/api/admin/sessions/revoke without a Bearer token → 401.
#[tokio::test]
async fn bulk_revoke_admin_sessions_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .post(get_admin_sessions_revoke_url())
        .json(&json!({ "session_ids": [Uuid::new_v4()] }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

// ── Security tests — non-admin user ──────────────────────────────────────────

/// GET /v1/api/admin/sessions with a regular user token → 403.
#[tokio::test]
async fn list_admin_sessions_with_non_admin_user_returns_403() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login(&client, &email, &password).await;

    let res = client
        .get(get_admin_sessions_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

/// DELETE /v1/api/admin/sessions/{id} with a regular user token → 403.
#[tokio::test]
async fn revoke_admin_session_with_non_admin_user_returns_403() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login(&client, &email, &password).await;

    let res = client
        .delete(get_admin_session_url(Uuid::new_v4()))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

/// POST /v1/api/admin/sessions/revoke with a regular user token → 403.
#[tokio::test]
async fn bulk_revoke_admin_sessions_with_non_admin_user_returns_403() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login(&client, &email, &password).await;

    let res = client
        .post(get_admin_sessions_revoke_url())
        .bearer_auth(&token)
        .json(&json!({ "session_ids": [Uuid::new_v4()] }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

// ── Input validation tests ────────────────────────────────────────────────────

/// POST /v1/api/admin/sessions/revoke with empty session_ids array → 400.
/// This is a pure input validation test — no admin token needed because the
/// middleware rejects unauth'd requests before validation. We use a regular
/// user token and expect 403, which still confirms the body reaches the
/// validation layer.  The canonical validation test is covered in the
/// admin happy-path block below.

/// POST /v1/api/admin/sessions/revoke without a Bearer token and empty
/// session_ids — server rejects auth before body validation, returns 401.
#[tokio::test]
async fn bulk_revoke_with_empty_session_ids_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .post(get_admin_sessions_revoke_url())
        .json(&json!({ "session_ids": [] }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    // Auth check fires before body validation
    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

// ── Admin happy-path tests ────────────────────────────────────────────────────

/// As admin: GET /v1/api/admin/sessions returns a list of sessions with the
/// expected fields (session_id, user_email, expires_at).
#[tokio::test]
async fn list_admin_sessions_as_admin_returns_session_list() {
    let client = Client::new();
    // Login to create at least one session, then list as admin
    let token = admin_login(&client).await;

    let res = client
        .get(get_admin_sessions_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200, got {status}\n{raw}");
    assert_eq!(body["success"], true, "{body}");

    let sessions = body["data"]
        .as_array()
        .unwrap_or_else(|| panic!("data must be array: {body}"));
    assert!(!sessions.is_empty(), "expected at least the admin's own session: {body}");

    let first = &sessions[0];
    assert!(first["session_id"].is_string(), "missing session_id: {body}");
    assert!(first["user_email"].is_string(), "missing user_email: {body}");
    assert!(first["expires_at"].is_string(), "missing expires_at: {body}");
}

/// As admin: DELETE /v1/api/admin/sessions/{id} revokes a known session.
/// We register a test user, log them in, look up their session via the user
/// sessions endpoint, then revoke it via the admin endpoint.
#[tokio::test]
async fn revoke_admin_session_as_admin_succeeds() {
    let client = Client::new();
    let admin_token = admin_login(&client).await;

    // Register a test user and log them in to create a session
    let email = format!("session-revoke-{}@example.com", Uuid::new_v4());
    let res = post_json(
        &client,
        get_register_user_url(),
        json!({
            "first_name": "Session",
            "last_name":  "Revoke",
            "email":      email,
            "password":   TEST_PASSWORD
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "register failed: {raw}");

    let res = post_json(
        &client,
        get_login_user_url(),
        json!({ "email": email, "password": TEST_PASSWORD }),
    )
    .await;
    let (status, login_body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");

    let user_token = login_body
        .pointer("/data/access_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing access_token: {login_body}"))
        .to_string();

    // Retrieve the user's own sessions to get a session_id
    let res = client
        .get(get_user_sessions_url())
        .bearer_auth(&user_token)
        .send()
        .await
        .expect("request failed");
    let (status, sessions_body, raw) = read_json(res).await;
    assert_eq!(status, 200, "get sessions failed: {raw}");

    let sessions = sessions_body["data"]
        .as_array()
        .unwrap_or_else(|| panic!("data must be array: {sessions_body}"));
    assert!(!sessions.is_empty(), "expected at least one session: {sessions_body}");

    let session_id_str = sessions[0]["session_id"]
        .as_str()
        .unwrap_or_else(|| panic!("missing session_id: {sessions_body}"));
    let session_id: Uuid = session_id_str.parse().expect("invalid session_id UUID");

    // Revoke via admin endpoint
    let res = client
        .delete(get_admin_session_url(session_id))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("request failed");
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "revoke session failed: {raw}");
}

/// As admin: DELETE /v1/api/admin/sessions/{id} with an unknown UUID → 404.
#[tokio::test]
async fn revoke_admin_session_unknown_id_returns_404() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let res = client
        .delete(get_admin_session_url(Uuid::new_v4()))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 404, "expected 404, got {status}\n{raw}");
    assert_error_envelope(&body, "NOT_FOUND");
}

/// As admin: POST /v1/api/admin/sessions/revoke with empty session_ids → 400.
#[tokio::test]
async fn bulk_revoke_admin_sessions_empty_ids_returns_400() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let res = client
        .post(get_admin_sessions_revoke_url())
        .bearer_auth(&token)
        .json(&json!({ "session_ids": [] }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

/// As admin: POST /v1/api/admin/sessions/revoke returns revoked_count equal
/// to the number of sessions actually found and revoked.
#[tokio::test]
async fn bulk_revoke_admin_sessions_as_admin_returns_revoked_count() {
    let client = Client::new();
    let admin_token = admin_login(&client).await;

    // Register a test user and create two sessions by logging in twice
    let email = format!("bulk-revoke-{}@example.com", Uuid::new_v4());
    let res = post_json(
        &client,
        get_register_user_url(),
        json!({
            "first_name": "Bulk",
            "last_name":  "Revoke",
            "email":      email,
            "password":   TEST_PASSWORD
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "register failed: {raw}");

    // First login — get the access token to look up sessions
    let res = post_json(
        &client,
        get_login_user_url(),
        json!({ "email": email, "password": TEST_PASSWORD }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "first login failed: {raw}");
    let user_token = body["data"]["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("missing access_token: {body}"))
        .to_string();

    // Second login — creates a second session
    let res = post_json(
        &client,
        get_login_user_url(),
        json!({ "email": email, "password": TEST_PASSWORD }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "second login failed: {raw}");

    // Retrieve all sessions for this user via their own endpoint
    let res = client
        .get(get_user_sessions_url())
        .bearer_auth(&user_token)
        .send()
        .await
        .expect("request failed");
    let (status, sessions_body, raw) = read_json(res).await;
    assert_eq!(status, 200, "get sessions failed: {raw}");

    let sessions = sessions_body["data"]
        .as_array()
        .unwrap_or_else(|| panic!("data must be array: {sessions_body}"));
    assert!(sessions.len() >= 2, "expected at least 2 sessions: {sessions_body}");

    let ids: Vec<serde_json::Value> = sessions
        .iter()
        .map(|s| json!(s["session_id"].as_str().expect("missing session_id")))
        .collect();
    let expected_count = ids.len() as u64;

    // Bulk revoke all sessions for this user
    let res = client
        .post(get_admin_sessions_revoke_url())
        .bearer_auth(&admin_token)
        .json(&json!({ "session_ids": ids }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "bulk revoke failed: {raw}");
    assert_eq!(body["success"], true, "{body}");
    assert_eq!(
        body["data"]["revoked_count"].as_u64(),
        Some(expected_count),
        "expected revoked_count={expected_count}: {body}"
    );
}
