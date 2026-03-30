mod common;

use common::{
    helpers::{assert_error_envelope, post_json, read_json},
    setup::{
        get_authenticate_token_url, get_login_user_url, get_register_user_url,
        get_user_permissions_url, get_user_session_url, get_user_sessions_url,
    },
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

/// Register a fresh user, log in, and return (access_token, session_id).
async fn register_login_authenticate(client: &Client) -> (String, Uuid) {
    let email = format!("sessions-{}@test.com", Uuid::new_v4());

    let res = post_json(
        client,
        get_register_user_url(),
        json!({
            "first_name": "Sessions",
            "last_name": "Tester",
            "email": email,
            "password": "T3stP@ssw0rd#Sec"
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "registration failed: {raw}");

    let res = post_json(
        client,
        get_login_user_url(),
        json!({ "email": email, "password": "T3stP@ssw0rd#Sec" }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");
    let token = body["data"]["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("no access_token: {raw}"))
        .to_string();

    // Authenticate to get session_id
    let res = client
        .post(get_authenticate_token_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("authenticate request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "authenticate failed: {raw}");
    let session_id = body["data"]["session_id"]
        .as_str()
        .unwrap_or_else(|| panic!("no session_id: {raw}"));
    let session_id = Uuid::parse_str(session_id).expect("invalid session_id UUID");

    (token, session_id)
}

// ── GET /v1/api/user/sessions ───────────────────────────────────────────────

#[tokio::test]
async fn get_sessions_returns_401_without_token() {
    let client = Client::new();
    let res = client
        .get(get_user_sessions_url())
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 401, "expected 401: {raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

#[tokio::test]
async fn get_sessions_returns_active_sessions() {
    let client = Client::new();
    let (token, session_id) = register_login_authenticate(&client).await;

    let res = client
        .get(get_user_sessions_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "expected 200: {raw}");
    assert_eq!(body["success"], true);

    let sessions = body["data"].as_array().expect("data must be array");
    assert!(!sessions.is_empty(), "expected at least one session");

    // Find the current session
    let current = sessions
        .iter()
        .find(|s| s["session_id"].as_str() == Some(&session_id.to_string()));
    assert!(current.is_some(), "current session not in list");
    assert_eq!(current.unwrap()["is_current"], true);

    // refresh_token_hash must not appear
    for s in sessions {
        assert!(
            s.get("refresh_token_hash").is_none(),
            "refresh_token_hash must not be exposed"
        );
    }
}

// ── GET /v1/api/user/sessions/{session_id} ──────────────────────────────────

#[tokio::test]
async fn get_session_detail_returns_401_without_token() {
    let client = Client::new();
    let res = client
        .get(get_user_session_url(Uuid::new_v4()))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 401, "expected 401: {raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

#[tokio::test]
async fn get_session_detail_returns_404_for_unknown_id() {
    let client = Client::new();
    let (token, _) = register_login_authenticate(&client).await;

    let res = client
        .get(get_user_session_url(Uuid::new_v4()))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 404, "expected 404: {raw}");
    assert_error_envelope(&body, "NOT_FOUND");
}

#[tokio::test]
async fn get_session_detail_returns_detail() {
    let client = Client::new();
    let (token, session_id) = register_login_authenticate(&client).await;

    let res = client
        .get(get_user_session_url(session_id))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "expected 200: {raw}");
    assert_eq!(body["success"], true);

    let data = &body["data"];
    assert_eq!(
        data["session_id"].as_str(),
        Some(session_id.to_string().as_str()),
        "session_id mismatch: {body}"
    );
    assert_eq!(data["is_current"], true, "expected is_current=true: {body}");
    assert_eq!(data["is_active"], true, "expected is_active=true: {body}");
    assert!(data["expires_at"].as_str().is_some(), "missing expires_at");
    assert!(
        data.get("refresh_token_hash").is_none(),
        "refresh_token_hash must not be exposed"
    );
}

#[tokio::test]
async fn get_session_detail_returns_404_for_another_users_session() {
    let client = Client::new();
    // Register two users
    let (token1, _) = register_login_authenticate(&client).await;
    let (_, session_id2) = register_login_authenticate(&client).await;

    // User 1 tries to access User 2's session
    let res = client
        .get(get_user_session_url(session_id2))
        .bearer_auth(&token1)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 404, "expected 404 for other user's session: {raw}");
    assert_error_envelope(&body, "NOT_FOUND");
}

// ── GET /v1/api/user/permissions ────────────────────────────────────────────

#[tokio::test]
async fn get_permissions_returns_401_without_token() {
    let client = Client::new();
    let res = client
        .get(get_user_permissions_url())
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 401, "expected 401: {raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

#[tokio::test]
async fn get_permissions_returns_user_roles() {
    let client = Client::new();
    let (token, _) = register_login_authenticate(&client).await;

    let res = client
        .get(get_user_permissions_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "expected 200: {raw}");
    assert_eq!(body["success"], true);

    let data = &body["data"];
    assert!(data["user_id"].as_str().is_some(), "missing user_id: {body}");

    let roles = data["roles"].as_array().expect("roles must be array");
    assert!(!roles.is_empty(), "expected at least one role");

    let first = &roles[0];
    assert!(first["role_id"].as_str().is_some(), "missing role_id");
    assert!(first["name"].as_str().is_some(), "missing name");
    assert!(first["role_type"].as_str().is_some(), "missing role_type");
    assert!(first["description"].as_str().is_some(), "missing description");
}
