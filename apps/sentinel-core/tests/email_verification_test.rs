mod common;

use common::{
    helpers::{assert_error_envelope, post_json, read_json},
    setup::{get_register_user_url, get_server_url},
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

fn get_verify_email_url(token: &str) -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/verify-email?token={token}")
}

fn get_resend_verification_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/resend-verification")
}

/// An invalid `ev_*` token must return 401 AUTH_ERROR (not leak token existence).
#[tokio::test]
async fn verify_email_with_invalid_token_returns_401() {
    let client = Client::new();

    let res = client
        .get(get_verify_email_url(
            "ev_badbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbad",
        ))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401 for invalid token: {raw}");
    assert_error_envelope(&body, "AUTH_ERROR");
}

/// An unknown email to resend-verification must return 404 NOT_FOUND.
#[tokio::test]
async fn resend_verification_with_unknown_email_returns_404() {
    let client = Client::new();

    let res = post_json(
        &client,
        get_resend_verification_url(),
        json!({ "email": format!("nonexistent-{}@example.com", Uuid::new_v4()) }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 404, "expected 404 for unknown email: {raw}");
    assert_error_envelope(&body, "NOT_FOUND");
}

/// A newly registered user whose email is unverified should not be able to access
/// the canary endpoint (which goes through authorize_middleware).
/// This test registers a user, logs in, and confirms the token has ev=false embedded
/// by checking that the authenticate endpoint reports email_verified=false.
#[tokio::test]
async fn newly_registered_user_token_has_email_verified_false() {
    let client = Client::new();
    let email = format!("evtest-{}@test.com", Uuid::new_v4());

    // Register
    let res = post_json(
        &client,
        get_register_user_url(),
        json!({
            "first_name": "Email",
            "last_name": "Test",
            "email": email,
            "password": "T3stP@ssw0rd#Sec"
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "registration failed: {raw}");

    // Login
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

    // Authenticate — check email_verified claim
    let res = client
        .post(format!("{}/v1/api/auth/authenticate", get_server_url()))
        .bearer_auth(&token)
        .send()
        .await
        .expect("authenticate request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "authenticate failed: {raw}");

    let email_verified = body["data"]["email_verified"]
        .as_bool()
        .unwrap_or_else(|| panic!("missing email_verified in response: {raw}"));
    assert!(
        !email_verified,
        "newly registered user should have email_verified=false: {raw}"
    );
}

/// Happy-path email flow tests are ignored because they require a live SMTP server.
#[tokio::test]
#[ignore]
async fn verify_email_happy_path_requires_smtp() {
    // Would need:
    // 1. Register user
    // 2. Intercept the raw token from the email (or insert directly into DB)
    // 3. Call GET /verify-email?token=<raw>
    // 4. Assert 200 and email_verified=true on subsequent authenticate
}
