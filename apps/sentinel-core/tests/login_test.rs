mod common;

use common::{
    helpers::{assert_error_envelope, post_json, read_json},
    setup::{get_login_user_url, get_register_user_url},
};
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

#[tokio::test]
async fn login_returns_validation_error_with_details() {
    let client = Client::new();

    let payload = json!({
        "email": "john.doeexample.com",
        "password": "anypass"
    });

    let res = post_json(&client, get_login_user_url(), payload).await;

    assert!(
        res.status().as_u16() == 400 || res.status().as_u16() == 422,
        "expected 400/422, got {}",
        res.status()
    );

    let body_text = res.text().await.unwrap_or_default();
    let body: Value = serde_json::from_str(&body_text).unwrap();

    assert_eq!(body["success"].as_bool(), Some(false), "{body}");
    assert!(body["data"].is_null(), "expected data=null: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("VALIDATION_ERROR"),
        "{body}"
    );

    let details = &body["error"]["details"];

    assert_eq!(
        details["email"][0]["code"].as_str(),
        Some("email"),
        "email validation missing/changed: {body}"
    );
}

#[tokio::test]
async fn login_with_nonexistent_user_returns_auth_error() {
    let client = Client::new();

    let email = format!("missing-{}@example.com", Uuid::new_v4());

    let payload = json!({
        "email": email,
        "password": "T3stP@ssw0rd#Sec"
    });

    let res = post_json(&client, get_login_user_url(), payload).await;
    let (status, body, raw) = read_json(res).await;

    assert!(
        status == 401 || status == 400,
        "expected 401/400, got {status}\nbody:\n{raw}"
    );

    let msg = assert_error_envelope(&body, "AUTH_ERROR");

    assert!(
        msg.to_lowercase().contains("invalid") || msg.to_lowercase().contains("credentials"),
        "unexpected message: {msg}"
    );
}

#[tokio::test]
async fn login_with_wrong_password_returns_auth_error() {
    let client = Client::new();

    // This assumes the user already exists in the DB
    let payload = json!({
        "email": "existing-user@example.com",
        "password": "wrong-password"
    });

    let res = post_json(&client, get_login_user_url(), payload).await;
    let (status, body, raw) = read_json(res).await;

    assert!(
        status == 401 || status == 400,
        "expected 401/400, got {status}\nbody:\n{raw}"
    );

    let msg = assert_error_envelope(&body, "AUTH_ERROR");

    assert!(
        msg.to_lowercase().contains("invalid") || msg.to_lowercase().contains("credentials"),
        "unexpected message: {msg}"
    );
}

#[tokio::test]
async fn login_success_returns_tokens() {
    let client = Client::new();

    // Create user with basic auth
    let email = format!("user-{}@example.com", Uuid::new_v4());
    let payload = json!({
        "first_name": "John",
        "last_name": "Doe",
        "email": email,
        "avatar_url": null,
        "password": "T3stP@ssw0rd#Sec"
    });

    // First call should succeed
    let res1 = post_json(&client, get_register_user_url(), payload.clone()).await;
    let (status1, body1, raw1) = read_json(res1).await;
    assert!(
        status1 == 200,
        "expected success status 200, got {status1}\nbody:\n{raw1}"
    );
    assert_eq!(
        body1.get("success").and_then(|v| v.as_bool()),
        Some(true),
        "{body1}"
    );

    // This assumes the user already exists in the DB
    let payload = json!({
        "email": email,
        "password": "T3stP@ssw0rd#Sec"
    });

    let res = post_json(&client, get_login_user_url(), payload).await;
    let (status, body, raw) = read_json(res).await;

    assert!(status == 200, "expected 200, got {status}\nbody:\n{raw}");
    assert_eq!(
        body.get("success").and_then(|v| v.as_bool()),
        Some(true),
        "{body}"
    );

    let data = body
        .get("data")
        .unwrap_or_else(|| panic!("missing data: {body}"));
    assert!(!data.is_null(), "expected data != null: {body}");

    let access_token = data
        .get("access_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing access_token: {body}"));

    assert!(
        access_token.len() > 20,
        "access_token too short: {access_token}"
    );
}
