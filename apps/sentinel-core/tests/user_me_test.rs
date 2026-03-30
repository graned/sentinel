mod common;

use common::{
    helpers::{assert_error_envelope, assert_login_success_envelope, post_json, read_json},
    setup::{get_login_user_url, get_register_user_url, get_user_me_url},
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

/// Register a fresh user, login, and return the access token + registration info.
async fn register_and_login(client: &Client) -> (String, String, String) {
    let email = format!("me-{}@test.com", Uuid::new_v4());
    let first_name = "Profile";
    let last_name = "User";

    let res = post_json(
        client,
        get_register_user_url(),
        json!({
            "first_name": first_name,
            "last_name": last_name,
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

    let token = body["data"]["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("no access_token: {raw}"))
        .to_string();

    (token, email, first_name.to_string())
}

/// GET /v1/api/user/me without a token → 401
#[tokio::test]
async fn me_returns_401_without_token() {
    let client = Client::new();

    let res = client
        .get(get_user_me_url())
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401 without token: {raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// GET /v1/api/user/me with a valid token → 200 with correct profile fields
#[tokio::test]
async fn me_returns_profile_for_authenticated_user() {
    let client = Client::new();
    let (token, email, first_name) = register_and_login(&client).await;

    let res = client
        .get(get_user_me_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200 with valid token: {raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = &body["data"];
    assert!(!data.is_null(), "data should not be null: {body}");

    assert!(
        data["user_id"].as_str().is_some(),
        "missing user_id: {body}"
    );
    assert_eq!(
        data["email"].as_str().unwrap_or(""),
        email,
        "email mismatch: {body}"
    );
    assert_eq!(
        data["first_name"].as_str().unwrap_or(""),
        first_name,
        "first_name mismatch: {body}"
    );
    assert!(data["status"].as_str().is_some(), "missing status: {body}");

    // password must never appear in the response
    assert!(
        data.get("password").is_none() && data.get("password_hash").is_none(),
        "password must not be exposed: {body}"
    );
}
