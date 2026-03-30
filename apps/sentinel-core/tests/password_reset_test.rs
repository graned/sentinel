mod common;

use common::{
    helpers::{assert_error_envelope, post_json, read_json},
    setup::{
        get_change_password_url, get_forgot_password_url, get_register_user_url,
        get_reset_password_url, get_server_url,
    },
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helper: register a user and optionally pre-verify their email in the DB.
// Returns the email address used.
// ---------------------------------------------------------------------------
async fn register_user(client: &Client) -> (String, String) {
    let email = format!("pwtest-{}@test.com", Uuid::new_v4());
    let password = "T3stP@ssw0rd#Sec".to_string();
    let res = post_json(
        client,
        get_register_user_url(),
        json!({
            "first_name": "Password",
            "last_name":  "Test",
            "email":      email,
            "password":   password,
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "registration failed: {raw}");
    (email, password)
}

/// Pre-verify a user's email directly via DB (bypasses SMTP requirement).
async fn mark_email_verified(email: &str) {
    use dotenvy::dotenv;
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
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
            "UPDATE user_identities SET email_verified = true WHERE email = $1",
            &[&email],
        )
        .await
        .expect("DB update failed");
}

// ---------------------------------------------------------------------------
// Forgot password tests
// ---------------------------------------------------------------------------

/// POST /password/forgot with unknown email must return 200 (anti-enumeration).
#[tokio::test]
async fn forgot_password_unknown_email_returns_200() {
    let client = Client::new();
    let res = post_json(
        &client,
        get_forgot_password_url(),
        json!({ "email": format!("nobody-{}@doesnotexist.example", Uuid::new_v4()) }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "expected 200 for unknown email: {raw}");
}

/// POST /password/forgot with an invalid email format must return 400.
#[tokio::test]
async fn forgot_password_invalid_email_format_returns_400() {
    let client = Client::new();
    let res = post_json(
        &client,
        get_forgot_password_url(),
        json!({ "email": "not-an-email" }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 400, "expected 400 for invalid email: {raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

// ---------------------------------------------------------------------------
// Reset password tests
// ---------------------------------------------------------------------------

/// POST /password/reset with a bogus token must return 401.
#[tokio::test]
async fn reset_password_invalid_token_returns_401() {
    let client = Client::new();
    let res = post_json(
        &client,
        get_reset_password_url(),
        json!({
            "token":        "pr_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "new_password": "T3stP@ssw0rd#Sec",
        }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 401, "expected 401 for invalid token: {raw}");
    assert_error_envelope(&body, "AUTH_ERROR");
}

/// POST /password/reset with a too-short password must return 400.
#[tokio::test]
async fn reset_password_weak_password_returns_400() {
    let client = Client::new();
    let res = post_json(
        &client,
        get_reset_password_url(),
        json!({
            "token":        "pr_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "new_password": "abc",
        }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 400, "expected 400 for weak password: {raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

// ---------------------------------------------------------------------------
// Change password tests
// ---------------------------------------------------------------------------

/// POST /user/password/change without a Bearer token must return 401.
#[tokio::test]
async fn change_password_without_auth_returns_401() {
    let client = Client::new();
    let res = post_json(
        &client,
        get_change_password_url(),
        json!({
            "current_password": "whatever",
            "new_password":     "newpassword123",
        }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 401, "expected 401 without auth: {raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// POST /user/password/change with the wrong current password must return 401.
#[tokio::test]
async fn change_password_wrong_current_returns_401() {
    let client = Client::new();
    let (email, _) = register_user(&client).await;
    mark_email_verified(&email).await;

    // Login to get token (must be after email verification so ev=true is baked in)
    let res = post_json(
        &client,
        format!("{}/v1/api/auth/login", get_server_url()),
        json!({ "email": email, "password": "T3stP@ssw0rd#Sec" }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");
    let token = body["data"]["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("no access_token: {raw}"))
        .to_string();

    // Attempt change with wrong current password
    let res = client
        .post(get_change_password_url())
        .bearer_auth(&token)
        .json(&json!({
            "current_password": "wrongpassword",
            "new_password":     "NewP@ssw0rd!Sec",
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(
        status, 401,
        "expected 401 for wrong current password: {raw}"
    );
    assert_error_envelope(&body, "AUTH_ERROR");
}

/// POST /user/password/change with a too-short new password must return 400.
#[tokio::test]
async fn change_password_weak_new_password_returns_400() {
    let client = Client::new();
    let (email, _) = register_user(&client).await;
    mark_email_verified(&email).await;

    let res = post_json(
        &client,
        format!("{}/v1/api/auth/login", get_server_url()),
        json!({ "email": email, "password": "T3stP@ssw0rd#Sec" }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");
    let token = body["data"]["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("no access_token: {raw}"))
        .to_string();

    let res = client
        .post(get_change_password_url())
        .bearer_auth(&token)
        .json(&json!({
            "current_password": "password123",
            "new_password":     "abc",
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 400, "expected 400 for weak password: {raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

// ---------------------------------------------------------------------------
// Happy-path tests (ignored — require live SMTP)
// ---------------------------------------------------------------------------

/// Full forgot → reset flow requires SMTP to receive the reset link.
#[tokio::test]
#[ignore]
async fn password_reset_happy_path() {
    // Would need:
    // 1. Register user
    // 2. POST /password/forgot
    // 3. Intercept raw token from email (or insert directly into DB)
    // 4. POST /password/reset with the token + new_password
    // 5. Assert 200 and old password no longer works
}

/// Full change password happy-path.
#[tokio::test]
#[ignore]
async fn change_password_happy_path() {
    // Would need:
    // 1. Register + verify email
    // 2. Login → get token
    // 3. POST /user/password/change with correct current_password
    // 4. Assert 200, old token still works until revoked, new credentials work
}
