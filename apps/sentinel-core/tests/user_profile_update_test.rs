//! Integration tests for PATCH /v1/api/user/me (profile update).
//!
//! Tests cover:
//! - Successful full profile update
//! - Partial update (single field)
//! - Empty body (no-op update)
//! - 401 without authentication
//! - Validation errors (field too long)

mod common;

use common::{
    helpers::{assert_error_envelope, post_json, read_json},
    setup::{get_login_user_url, get_register_user_url, get_user_me_url},
};
use reqwest::Client;
use serde_json::json;
use std::env;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helper: register + pre-verify email + login, returning the access token.
// ---------------------------------------------------------------------------

async fn register_verified_and_login(client: &Client) -> (String, String) {
    let email = format!("profile-{}@test.com", Uuid::new_v4());

    // Register
    let res = post_json(
        client,
        get_register_user_url(),
        json!({
            "first_name": "Original",
            "last_name":  "Name",
            "email":      email,
            "password":   "T3stP@ssw0rd#Sec",
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "registration failed: {raw}");

    // Pre-verify email in DB so authorize_middleware doesn't block us
    mark_email_verified(&email).await;

    // Login
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

    (token, email)
}

/// Pre-verify a user's email directly via tokio-postgres.
/// See gotcha #13: authorize_middleware blocks unverified users, so tests
/// that hit protected endpoints must pre-verify the email before login.
async fn mark_email_verified(email: &str) {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
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
// Test: full profile update with all fields
// ---------------------------------------------------------------------------

/// PATCH /v1/api/user/me with first_name, last_name, and avatar_url
/// returns 200 with updated profile fields.
#[tokio::test]
async fn update_profile_with_all_fields_returns_200() {
    let client = Client::new();
    let (token, _email) = register_verified_and_login(&client).await;

    let new_first_name = "Updated";
    let new_last_name = "Person";
    let new_avatar = "https://example.com/avatar.png";

    let res = client
        .patch(get_user_me_url())
        .bearer_auth(&token)
        .json(&json!({
            "first_name": new_first_name,
            "last_name": new_last_name,
            "avatar_url": new_avatar
        }))
        .send()
        .await
        .expect("HTTP request failed");

    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "expected 200 with valid update: {raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = &body["data"];
    assert_eq!(
        data["first_name"].as_str().unwrap_or(""),
        new_first_name,
        "first_name should be updated: {body}"
    );
    assert_eq!(
        data["last_name"].as_str().unwrap_or(""),
        new_last_name,
        "last_name should be updated: {body}"
    );
    assert_eq!(
        data["avatar_url"].as_str().unwrap_or(""),
        new_avatar,
        "avatar_url should be updated: {body}"
    );

    // Verify the changes persist by fetching the profile again
    let res = client
        .get(get_user_me_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("HTTP request failed");
    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 200, "GET after PATCH should work");
    assert_eq!(
        body["data"]["first_name"].as_str().unwrap_or(""),
        new_first_name,
        "first_name should persist: {body}"
    );
}

// ---------------------------------------------------------------------------
// Test: partial update (only first_name)
// ---------------------------------------------------------------------------

/// PATCH /v1/api/user/me with only first_name updates first_name
/// while leaving last_name and avatar_url unchanged.
#[tokio::test]
async fn update_profile_partial_update_returns_200() {
    let client = Client::new();
    let (token, _email) = register_verified_and_login(&client).await;

    // Partial update: only first_name
    let new_first_name = "Partial";
    let res = client
        .patch(get_user_me_url())
        .bearer_auth(&token)
        .json(&json!({
            "first_name": new_first_name
        }))
        .send()
        .await
        .expect("HTTP request failed");

    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "expected 200 with partial update: {raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = &body["data"];
    assert_eq!(
        data["first_name"].as_str().unwrap_or(""),
        new_first_name,
        "first_name should be updated: {body}"
    );
    // last_name and avatar_url should remain as they were (from registration)
    assert_eq!(
        data["last_name"].as_str().unwrap_or(""),
        "Name",
        "last_name should be unchanged: {body}"
    );
    assert!(
        data["avatar_url"].is_null() || data["avatar_url"].as_str().is_none(),
        "avatar_url should be null/empty when not set: {body}"
    );
}

// ---------------------------------------------------------------------------
// Test: empty body (no-op update)
// ---------------------------------------------------------------------------

/// PATCH /v1/api/user/me with an empty body returns 200
/// and leaves all profile fields unchanged.
#[tokio::test]
async fn update_profile_empty_body_returns_200() {
    let client = Client::new();
    let (token, _email) = register_verified_and_login(&client).await;

    let res = client
        .patch(get_user_me_url())
        .bearer_auth(&token)
        .json(&json!({}))
        .send()
        .await
        .expect("HTTP request failed");

    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "expected 200 with empty body: {raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = &body["data"];
    // Profile should remain unchanged from registration
    assert_eq!(
        data["first_name"].as_str().unwrap_or(""),
        "Original",
        "first_name should be unchanged: {body}"
    );
    assert_eq!(
        data["last_name"].as_str().unwrap_or(""),
        "Name",
        "last_name should be unchanged: {body}"
    );
}

// ---------------------------------------------------------------------------
// Test: missing authentication
// ---------------------------------------------------------------------------

/// PATCH /v1/api/user/me without a token returns 401.
#[tokio::test]
async fn update_profile_without_token_returns_401() {
    let client = Client::new();

    let res = client
        .patch(get_user_me_url())
        .json(&json!({ "first_name": "Test" }))
        .send()
        .await
        .expect("HTTP request failed");

    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 401, "expected 401 without token: {raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

// ---------------------------------------------------------------------------
// Test: invalid token
// ---------------------------------------------------------------------------

/// PATCH /v1/api/user/me with an invalid token returns 401.
#[tokio::test]
async fn update_profile_with_invalid_token_returns_401() {
    let client = Client::new();

    let res = client
        .patch(get_user_me_url())
        .bearer_auth("invalid-token-value")
        .json(&json!({ "first_name": "Test" }))
        .send()
        .await
        .expect("HTTP request failed");

    let (status, _body, raw) = read_json(res).await;
    assert_eq!(status, 401, "expected 401 with invalid token: {raw}");
}

// ---------------------------------------------------------------------------
// Test: validation - first_name too long
// ---------------------------------------------------------------------------

/// PATCH /v1/api/user/me with first_name exceeding 100 characters
/// returns 400 VALIDATION_ERROR.
#[tokio::test]
async fn update_profile_first_name_too_long_returns_400() {
    let client = Client::new();
    let (token, _email) = register_verified_and_login(&client).await;

    let too_long_name = "a".repeat(101);

    let res = client
        .patch(get_user_me_url())
        .bearer_auth(&token)
        .json(&json!({
            "first_name": too_long_name
        }))
        .send()
        .await
        .expect("HTTP request failed");

    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 400, "expected 400 for too-long first_name: {raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

// ---------------------------------------------------------------------------
// Test: validation - last_name too long
// ---------------------------------------------------------------------------

/// PATCH /v1/api/user/me with last_name exceeding 100 characters
/// returns 400 VALIDATION_ERROR.
#[tokio::test]
async fn update_profile_last_name_too_long_returns_400() {
    let client = Client::new();
    let (token, _email) = register_verified_and_login(&client).await;

    let too_long_name = "b".repeat(101);

    let res = client
        .patch(get_user_me_url())
        .bearer_auth(&token)
        .json(&json!({
            "last_name": too_long_name
        }))
        .send()
        .await
        .expect("HTTP request failed");

    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 400, "expected 400 for too-long last_name: {raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

// ---------------------------------------------------------------------------
// Test: response shape
// ---------------------------------------------------------------------------

/// PATCH /v1/api/user/me returns the full UserProfileResponse shape
/// including user_id, email, status, etc.
#[tokio::test]
async fn update_profile_returns_full_user_profile_response() {
    let client = Client::new();
    let (token, email) = register_verified_and_login(&client).await;

    let res = client
        .patch(get_user_me_url())
        .bearer_auth(&token)
        .json(&json!({
            "first_name": "Response",
            "last_name": "Check"
        }))
        .send()
        .await
        .expect("HTTP request failed");

    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "expected 200: {raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = &body["data"];

    // Required fields in UserProfileResponse
    assert!(data["user_id"].is_string(), "user_id must be a string: {body}");
    assert_eq!(data["email"].as_str().unwrap_or(""), email, "email mismatch: {body}");
    assert!(data["first_name"].is_string(), "first_name must be a string: {body}");
    assert!(data["last_name"].is_string(), "last_name must be a string: {body}");
    assert!(data["status"].is_string(), "status must be a string: {body}");
    assert!(data["email_verified"].is_boolean(), "email_verified must be boolean: {body}");
    assert!(data["mfa_enabled"].is_boolean(), "mfa_enabled must be boolean: {body}");
    // avatar_url is optional
    assert!(
        data["avatar_url"].is_string() || data["avatar_url"].is_null(),
        "avatar_url must be string or null: {body}"
    );

    // Password must never appear in the response
    assert!(
        data.get("password").is_none() && data.get("password_hash").is_none(),
        "password must not be exposed: {body}"
    );
}