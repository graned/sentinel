mod common;

use common::{
    helpers::{generate_expired_token, post_json, read_json},
    setup::{get_authenticate_token_url, get_login_user_url, get_register_user_url},
};
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

#[tokio::test]
async fn authenticate_without_token_returns_401() {
    let client = Client::new();

    let res = client
        .post(get_authenticate_token_url())
        .header("Authorization", "")
        .send()
        .await
        .expect("request failed");

    assert_eq!(res.status(), 401);

    let body: Value = res.json().await.expect("response must be JSON");

    assert_eq!(body["success"], false);
    assert!(body["data"].is_null(), "data should be null on error");
    assert!(
        body["error"].is_object(),
        "error should be present on error"
    );

    // Optional strictness: enforce your error code (adjust to your implementation)
    assert_eq!(body["error"]["code"], "MISSING_TOKEN");
}

#[tokio::test]
async fn authenticate_with_malformed_token_returns_401() {
    let client = Client::new();

    let res = client
        .post(get_authenticate_token_url())
        .header("Authorization", "Bearer definitely-not-a-real-token")
        .send()
        .await
        .expect("request failed");

    assert_eq!(res.status(), 401);

    let body: Value = res.json().await.expect("response must be JSON");

    assert_eq!(body["success"], false);
    assert!(body["data"].is_null(), "data should be null on error");
    assert!(
        body["error"].is_object(),
        "error should be present on error"
    );

    // Optional strictness: enforce your error code (adjust to your implementation)
    assert_eq!(body["error"]["code"], "INVALID_TOKEN");
}

#[tokio::test]
async fn authenticate_with_expired_token_returns_401() {
    let client = Client::new();

    // This token just needs to be recognized by your implementation
    // as "expired". If you currently don't distinguish, keep the test
    // but comment out the code assertion until implemented.
    let uid = Uuid::new_v4();
    let sid = Uuid::new_v4();
    let expired_token = generate_expired_token(uid, sid).unwrap();

    let res = client
        .post(get_authenticate_token_url())
        .header("Authorization", format!("Bearer {}", expired_token))
        .send()
        .await
        .expect("request failed");

    assert_eq!(res.status(), 401);

    let body: Value = res.json().await.expect("response must be JSON");

    assert_eq!(body["success"], false);
    assert!(body["data"].is_null());
    assert!(body["error"].is_object());

    // Recommended: distinguish expired vs invalid
    // Enable once implemented
    assert_eq!(body["error"]["code"], "EXPIRED_TOKEN");
}

#[tokio::test]
async fn authenticate_with_valid_token_returns_200() {
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
    let (status1, body1, _raw1) = read_json(res1).await;
    assert_eq!(status1, 200);
    assert_eq!(body1["success"], true);

    // login
    // This assumes the user already exists in the DB
    let payload = json!({
        "email": email,
        "password": "T3stP@ssw0rd#Sec"
    });

    let res2 = post_json(&client, get_login_user_url(), payload).await;
    let (status2, body2, _raw2) = read_json(res2).await;
    assert_eq!(status2, 200);
    assert_eq!(body2["success"], true);

    // test token
    let access_token = body2["data"]["access_token"]
        .as_str()
        .expect("login response must include data.access_token")
        .to_string();

    let res = client
        .post(get_authenticate_token_url())
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .expect("request failed");

    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.expect("response must be JSON");

    // Success semantics
    assert_eq!(body["success"], true);
    assert!(body["error"].is_null(), "error must be null on success");
    assert!(body["data"].is_object(), "data must be object on success");

    // Minimal, future-proof assertions
    let data = body["data"].as_object().unwrap();

    assert!(
        data.contains_key("user_id"),
        "authenticate success should include 'user_id'"
    );
    assert!(
        data.contains_key("session_id"),
        "authenticate success should include 'session_id'"
    );
}
