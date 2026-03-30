mod common;

use common::{
    helpers::{admin_login, assert_error_envelope, post_json, read_json},
    setup::{
        get_admin_user_status_url, get_admin_user_url, get_admin_users_url, get_login_user_url,
        get_register_user_url,
    },
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

// ── Setup helpers ─────────────────────────────────────────────────────────────

/// Register a fresh user and return `(email, password)`.
async fn register_user(client: &Client) -> (String, String) {
    let email = format!("admin-user-test-{}@example.com", Uuid::new_v4());
    let password = "T3stP@ssw0rd#Sec";

    let res = post_json(
        client,
        get_register_user_url(),
        json!({
            "first_name": "Test",
            "last_name":  "User",
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

// ── Security tests — no admin needed ─────────────────────────────────────────

/// GET /v1/api/admin/users without a Bearer token → 401.
#[tokio::test]
async fn list_admin_users_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .get(get_admin_users_url())
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// POST /v1/api/admin/users without a Bearer token → 401.
#[tokio::test]
async fn create_admin_user_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .post(get_admin_users_url())
        .json(&json!({
            "email": "new@example.com",
            "first_name": "New",
            "last_name": "User",
            "password": "T3stP@ssw0rd#Sec"
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// DELETE /v1/api/admin/users/{id} without a Bearer token → 401.
#[tokio::test]
async fn delete_admin_user_without_auth_returns_401() {
    let client = Client::new();

    let fake_id = Uuid::new_v4();
    let res = client
        .delete(get_admin_user_url(fake_id))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// PUT /v1/api/admin/users/{id}/status without a Bearer token → 401.
#[tokio::test]
async fn update_user_status_without_auth_returns_401() {
    let client = Client::new();

    let fake_id = Uuid::new_v4();
    let res = client
        .put(get_admin_user_status_url(fake_id))
        .json(&json!({ "status": "suspended" }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// A regular user (role = "user") cannot list users → 403.
#[tokio::test]
async fn list_admin_users_with_non_admin_returns_403() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login(&client, &email, &password).await;

    let res = client
        .get(get_admin_users_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

/// A regular user (role = "user") cannot create users → 403.
#[tokio::test]
async fn create_admin_user_with_non_admin_returns_403() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login(&client, &email, &password).await;

    let res = client
        .post(get_admin_users_url())
        .bearer_auth(&token)
        .json(&json!({
            "email": "another@example.com",
            "first_name": "New",
            "last_name": "User",
            "password": "T3stP@ssw0rd#Sec"
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

/// A regular user cannot delete users → 403.
#[tokio::test]
async fn delete_admin_user_with_non_admin_returns_403() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login(&client, &email, &password).await;

    let fake_id = Uuid::new_v4();
    let res = client
        .delete(get_admin_user_url(fake_id))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

/// A regular user cannot update user status → 403.
#[tokio::test]
async fn update_user_status_with_non_admin_returns_403() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login(&client, &email, &password).await;

    let fake_id = Uuid::new_v4();
    let res = client
        .put(get_admin_user_status_url(fake_id))
        .bearer_auth(&token)
        .json(&json!({ "status": "suspended" }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

// ── Admin happy-path tests ────────────────────────────────────────────────────

/// Admin can list users; response is paginated and includes at least the seeded admin.
#[tokio::test]
async fn admin_can_list_users() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let res = client
        .get(get_admin_users_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200, got {status}\n{raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = &body["data"];
    assert!(data["items"].is_array(), "data.items must be array: {body}");
    assert!(
        data["total"].is_number(),
        "data.total must be a number: {body}"
    );
    assert!(
        data["page"].is_number(),
        "data.page must be a number: {body}"
    );
    assert!(
        data["page_size"].is_number(),
        "data.page_size must be a number: {body}"
    );

    let total = data["total"].as_i64().unwrap_or(0);
    assert!(
        total >= 1,
        "should have at least the seeded admin user: {body}"
    );
}

/// Admin full lifecycle: create user → list contains new user → update status → delete.
#[tokio::test]
async fn admin_user_lifecycle() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let email = format!("lifecycle-user-{}@example.com", Uuid::new_v4());

    // Create
    let res = client
        .post(get_admin_users_url())
        .bearer_auth(&token)
        .json(&json!({
            "email": email,
            "first_name": "Lifecycle",
            "last_name": "User",
            "password": "T3stP@ssw0rd#Sec"
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "create user failed: {raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = &body["data"];
    let user_id_str = data["user_id"]
        .as_str()
        .unwrap_or_else(|| panic!("missing user_id: {body}"));
    let user_id: Uuid = user_id_str.parse().expect("user_id is not a valid UUID");
    assert_eq!(data["email"].as_str(), Some(email.as_str()), "{body}");
    // Newly created users start as pending_verification until email is confirmed
    assert!(
        matches!(
            data["status"].as_str(),
            Some("active") | Some("pending_verification")
        ),
        "unexpected status: {body}"
    );

    // Update status → suspended
    let res = client
        .put(get_admin_user_status_url(user_id))
        .bearer_auth(&token)
        .json(&json!({ "status": "suspended" }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "update status failed: {raw}");
    assert_eq!(body["data"]["status"].as_str(), Some("suspended"), "{body}");

    // Delete
    let res = client
        .delete(get_admin_user_url(user_id))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "delete user failed: {raw}");
}
