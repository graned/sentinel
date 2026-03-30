mod common;

use common::{
    helpers::{assert_error_envelope, assert_login_success_envelope, post_json, read_json},
    setup::{get_login_user_url, get_register_user_url, get_token_refresh_url},
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

/// Register a new user and log in, returning (access_token, refresh_token).
async fn register_and_login(client: &Client) -> (String, String) {
    let email = format!("refresh-{}@test.com", Uuid::new_v4());

    let res = post_json(
        client,
        get_register_user_url(),
        json!({
            "first_name": "Refresh",
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
    assert_login_success_envelope(&body);

    let access_token = body["data"]["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("no access_token: {raw}"))
        .to_string();
    let refresh_token = body["data"]["refresh_token"]
        .as_str()
        .unwrap_or_else(|| panic!("no refresh_token: {raw}"))
        .to_string();

    (access_token, refresh_token)
}

/// POST /v1/api/auth/token/refresh with a valid refresh token → 200, new token pair
#[tokio::test]
async fn token_refresh_returns_new_token_pair() {
    let client = Client::new();
    let (_access_token, refresh_token) = register_and_login(&client).await;

    let res = post_json(
        &client,
        get_token_refresh_url(),
        json!({ "refresh_token": refresh_token }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200 on valid refresh: {raw}");
    assert_eq!(body["success"], true, "{body}");

    let new_access = body["data"]["access_token"].as_str();
    let new_refresh = body["data"]["refresh_token"].as_str();
    assert!(new_access.is_some(), "missing access_token in response: {body}");
    assert!(new_refresh.is_some(), "missing refresh_token in response: {body}");

    // New tokens must differ from old ones
    assert_ne!(
        new_refresh.unwrap(),
        refresh_token,
        "refresh token should rotate"
    );
}

/// RTR: using the old refresh token after rotation → 401
#[tokio::test]
async fn token_refresh_old_token_rejected_after_rotation() {
    let client = Client::new();
    let (_access_token, refresh_token) = register_and_login(&client).await;

    // First refresh — consumes the original token
    let res = post_json(
        &client,
        get_token_refresh_url(),
        json!({ "refresh_token": refresh_token }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "first refresh should succeed: {raw}");

    // Second refresh with the now-consumed original token → 401
    let res = post_json(
        &client,
        get_token_refresh_url(),
        json!({ "refresh_token": refresh_token }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 401, "reused refresh token should be rejected: {raw}");
    assert_error_envelope(&body, "INVALID_TOKEN");
}

/// POST /v1/api/auth/token/refresh with a garbage token → 401
#[tokio::test]
async fn token_refresh_garbage_token_returns_401() {
    let client = Client::new();

    let res = post_json(
        &client,
        get_token_refresh_url(),
        json!({ "refresh_token": "rt_thisisnotarealtoken" }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "garbage token should return 401: {raw}");
    assert_error_envelope(&body, "INVALID_TOKEN");
}

/// POST /v1/api/auth/token/refresh with missing body field → 400
#[tokio::test]
async fn token_refresh_missing_field_returns_400() {
    let client = Client::new();

    let res = post_json(&client, get_token_refresh_url(), json!({})).await;
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "missing field should return 400: {raw}");
    assert_eq!(body["success"], false, "{body}");
}
