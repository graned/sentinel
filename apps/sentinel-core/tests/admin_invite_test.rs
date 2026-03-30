mod common;

use common::{
    helpers::{assert_error_envelope, post_json, read_json},
    setup::{
        get_admin_user_invite_link_url, get_admin_user_send_invite_url, get_admin_users_url,
        get_login_user_url, get_user_me_url,
    },
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Login as the seeded super admin and return a Bearer token.
async fn admin_login(client: &Client) -> String {
    disable_admin_mfa().await;
    let res = post_json(
        client,
        get_login_user_url(),
        json!({ "email": "admin@sentinel.local", "password": "admin" }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "admin login failed: {raw}");
    body.pointer("/data/access_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing access_token in admin login: {body}"))
        .to_string()
}

/// Admin creates a user and returns `(user_id, email, password)`.
async fn admin_create_user(client: &Client, admin_token: &str) -> (String, String, String) {
    let email = format!("invite-test-{}@example.com", Uuid::new_v4());
    let password = "TempP@ssw0rd#Inv";

    let res = client
        .post(get_admin_users_url())
        .bearer_auth(admin_token)
        .json(&json!({
            "email": email,
            "first_name": "Invite",
            "last_name": "Tester",
            "password": password,
        }))
        .send()
        .await
        .expect("request failed");

    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "admin create user failed: {raw}");

    let user_id = body
        .pointer("/data/user_id")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing user_id: {body}"))
        .to_string();

    (user_id, email, password.to_string())
}

// ── Tests — no admin user needed ──────────────────────────────────────────────

/// POST /v1/api/admin/users/{id}/send-invite without auth → 401.
#[tokio::test]
async fn send_invite_without_auth_returns_401() {
    let client = Client::new();
    let fake_id = Uuid::new_v4();
    let res = client
        .post(get_admin_user_send_invite_url(fake_id))
        .send()
        .await
        .expect("request failed");
    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 401);
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// GET /v1/api/admin/users/{id}/invite-link without auth → 401.
#[tokio::test]
async fn get_invite_link_without_auth_returns_401() {
    let client = Client::new();
    let fake_id = Uuid::new_v4();
    let res = client
        .get(get_admin_user_invite_link_url(fake_id))
        .send()
        .await
        .expect("request failed");
    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 401);
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// POST /v1/api/admin/users/{id}/send-invite with non-admin token → 403.
#[tokio::test]
async fn send_invite_with_non_admin_returns_403() {
    use common::setup::get_register_user_url;
    let client = Client::new();

    // Register + login as a regular user
    let email = format!("non-admin-invite-{}@example.com", Uuid::new_v4());
    let password = "T3stP@ssw0rd#Sec";
    let _ = post_json(
        &client,
        get_register_user_url(),
        json!({
            "first_name": "Non",
            "last_name":  "Admin",
            "email":      email,
            "password":   password
        }),
    )
    .await;
    let res = post_json(
        &client,
        get_login_user_url(),
        json!({ "email": email, "password": password }),
    )
    .await;
    let (_, body, _) = read_json(res).await;
    let token = body
        .pointer("/data/access_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let fake_id = Uuid::new_v4();
    let res = client
        .post(get_admin_user_send_invite_url(fake_id))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 403, "expected 403, body: {body}");
}

// ── Happy-path tests (require seeded admin) ───────────────────────────────────

/// Admin-created user has status `pending_verification` and email_verified false.
#[tokio::test]
async fn admin_create_user_status_is_pending_verification() {
    let client = Client::new();
    let admin_token = admin_login(&client).await;

    let email = format!("status-test-{}@example.com", Uuid::new_v4());
    let res = client
        .post(get_admin_users_url())
        .bearer_auth(&admin_token)
        .json(&json!({
            "email": email,
            "first_name": "Status",
            "last_name": "Test",
            "password": "TempP@ssw0rd#Inv",
        }))
        .send()
        .await
        .expect("request failed");

    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "create user failed: {raw}");

    let data = body.pointer("/data").expect("missing data");
    assert_eq!(
        data.pointer("/status").and_then(|v| v.as_str()),
        Some("pending_verification"),
        "status should be pending_verification, got: {data}"
    );
    assert_eq!(
        data.pointer("/email_verified").and_then(|v| v.as_bool()),
        Some(false),
        "email_verified should be false, got: {data}"
    );
}

/// Admin-created user login returns must_change_password = true.
#[tokio::test]
async fn admin_created_user_login_returns_must_change_password() {
    let client = Client::new();
    let admin_token = admin_login(&client).await;
    let (_, email, password) = admin_create_user(&client, &admin_token).await;

    // Manually verify email via DB so the user can log in (email gate)
    mark_email_verified(&email).await;

    let res = post_json(
        &client,
        get_login_user_url(),
        json!({ "email": email, "password": password }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");

    let must_change = body
        .pointer("/data/must_change_password")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| panic!("missing must_change_password in response: {body}"));
    assert!(must_change, "must_change_password should be true for admin-created user");
}

/// Token with mcp=true is blocked on /user/me with MUST_CHANGE_PASSWORD.
#[tokio::test]
async fn mcp_token_is_blocked_on_protected_route() {
    let client = Client::new();
    let admin_token = admin_login(&client).await;
    let (_, email, password) = admin_create_user(&client, &admin_token).await;

    mark_email_verified(&email).await;

    // Login to get mcp token
    let res = post_json(
        &client,
        get_login_user_url(),
        json!({ "email": email, "password": password }),
    )
    .await;
    let (_, body, raw) = read_json(res).await;
    let token = body
        .pointer("/data/access_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("no token: {raw}"))
        .to_string();

    // Attempt to access a protected route
    let res = client
        .get(get_user_me_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 403, "expected 403 MUST_CHANGE_PASSWORD, body: {body}");
    assert_error_envelope(&body, "MUST_CHANGE_PASSWORD");
}

/// GET /v1/api/admin/users/{id}/invite-link returns a URL containing /verify-email?token=
#[tokio::test]
async fn get_invite_link_returns_verify_email_url() {
    let client = Client::new();
    let admin_token = admin_login(&client).await;
    let (user_id, _, _) = admin_create_user(&client, &admin_token).await;
    let user_uuid = Uuid::parse_str(&user_id).expect("invalid uuid");

    let res = client
        .get(get_admin_user_invite_link_url(user_uuid))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("request failed");

    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "get invite link failed: {raw}");

    let invite_url = body
        .pointer("/data/invite_url")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing invite_url: {body}"));

    assert!(
        invite_url.contains("/verify-email?token="),
        "invite_url should contain /verify-email?token=, got: {invite_url}"
    );
    assert!(
        invite_url.starts_with("http"),
        "invite_url should be an absolute URL, got: {invite_url}"
    );
}

/// GET /v1/api/admin/users/{id}/invite-link for already-verified user → 400.
#[tokio::test]
async fn get_invite_link_for_verified_user_returns_400() {
    let client = Client::new();
    let admin_token = admin_login(&client).await;
    let (user_id, email, _) = admin_create_user(&client, &admin_token).await;
    let user_uuid = Uuid::parse_str(&user_id).expect("invalid uuid");

    // Pre-verify so the endpoint rejects
    mark_email_verified(&email).await;

    let res = client
        .get(get_admin_user_invite_link_url(user_uuid))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("request failed");

    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 400, "expected 400 for already-verified, got: {body}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

// ── DB helpers (tokio-postgres) ───────────────────────────────────────────────

async fn disable_admin_mfa() {
    use dotenvy::dotenv;
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set for test");
    let (client, connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
        .await
        .expect("DB connection failed");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("DB connection error: {e}");
        }
    });
    client
        .execute(
            "UPDATE user_mfa_totp SET enabled = false \
             WHERE user_id = (SELECT user_id FROM user_identities WHERE email = 'admin@sentinel.local')",
            &[],
        )
        .await
        .expect("DB update user_mfa_totp failed");
}

async fn mark_email_verified(email: &str) {
    use dotenvy::dotenv;
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set for test");
    let (client, connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
        .await
        .expect("DB connection failed");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("DB connection error: {e}");
        }
    });
    // Mark email verified and promote status to active so login works
    client
        .execute(
            "UPDATE user_identities SET email_verified = true WHERE email = $1",
            &[&email],
        )
        .await
        .expect("DB update user_identities failed");
    client
        .execute(
            "UPDATE users SET status = 'active' \
             WHERE user_id = (SELECT user_id FROM user_identities WHERE email = $1)",
            &[&email],
        )
        .await
        .expect("DB update users status failed");
}
