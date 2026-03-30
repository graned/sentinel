mod common;

use common::{
    helpers::{assert_error_envelope, post_json, read_json, TEST_PASSWORD},
    setup::get_register_user_url,
};
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

#[tokio::test]
async fn register_user_duplicate_email_returns_validation_error() {
    let client = Client::new();

    let email = format!("dup-{}@example.com", Uuid::new_v4());

    let payload = json!({
        "first_name": "John",
        "last_name": "Doe",
        "email": email,
        "avatar_url": null,
        "password": TEST_PASSWORD
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

    // Second call should fail with expected envelop response
    let res2 = post_json(&client, get_register_user_url(), payload.clone()).await;
    let (status2, body2, raw2) = read_json(res2).await;

    assert!(status2 == 400, "expected 400, got {status2}\nbody:\n{raw2}");

    let msg = assert_error_envelope(&body2, "VALIDATION_ERROR");
    assert!(
        msg.contains("already exists") && msg.contains(&email),
        "expected message to mention duplicate email. msg={msg}"
    );
}

#[tokio::test]
async fn register_returns_validation_error_with_details() {
    let client = Client::new();

    let payload = json!({
        "first_name": "",
        "last_name": "",
        "email": "john.doeexample.com",
        "password": "weak"
    });

    let res = post_json(&client, get_register_user_url(), payload.clone()).await;

    // Most APIs use 400 or 422 for validation; if yours is fixed, lock it down.
    assert!(
        res.status().as_u16() == 400 || res.status().as_u16() == 422,
        "expected 400/422, got {}",
        res.status()
    );

    let body_text = res.text().await.unwrap_or_default();
    let body: Value = serde_json::from_str(&body_text).unwrap_or_else(|e| {
        panic!("response was not JSON: {e}\nbody:\n{body_text}");
    });

    // Envelope assertions
    assert_eq!(body["success"].as_bool(), Some(false), "{body}");
    assert!(body["data"].is_null(), "expected data=null, got: {body}");
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("VALIDATION_ERROR"),
        "{body}"
    );
    assert_eq!(
        body["error"]["message"].as_str(),
        Some("Request validation failed"),
        "{body}"
    );
    assert!(body.get("timestamp").is_some(), "missing timestamp: {body}");
    assert!(
        body.get("request_id").is_some(),
        "missing request_id: {body}"
    );

    // Details assertions (exactly like your sample)
    let details = &body["error"]["details"];

    // email -> first error -> code/message
    assert_eq!(
        details["email"][0]["code"].as_str(),
        Some("email"),
        "email validation missing/changed: {body}"
    );
    assert_eq!(
        details["email"][0]["message"].as_str(),
        Some("Invalid email format"),
        "email validation message missing/changed: {body}"
    );
    assert_eq!(
        details["email"][0]["params"]["value"].as_str(),
        Some("john.doeexample.com"),
        "email params.value missing/changed: {body}"
    );

    // first_name length
    assert_eq!(
        details["first_name"][0]["code"].as_str(),
        Some("length"),
        "first_name validation missing/changed: {body}"
    );

    // last_name length
    assert_eq!(
        details["last_name"][0]["code"].as_str(),
        Some("length"),
        "last_name validation missing/changed: {body}"
    );

    // password custom validation (too short / missing requirements)
    assert!(
        details["password"][0]["code"].as_str().is_some(),
        "password validation missing: {body}"
    );
}
