mod common;

use common::{
    helpers::{admin_login, assert_error_envelope, post_json, read_json},
    setup::get_create_policy_url,
};
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn valid_rules() -> Value {
    json!([
        { "method": "GET",    "path": "/users/:id", "roles": ["user", "admin"] },
        { "method": "DELETE", "path": "/users/**",  "roles": ["admin"] }
    ])
}

fn assert_is_uuid(v: &Value, path: &str) {
    let s = v
        .as_str()
        .unwrap_or_else(|| panic!("{path} must be a string, got: {v}"));
    Uuid::parse_str(s).unwrap_or_else(|_| panic!("{path} must be a valid UUID, got: {s}"));
}

// ── Success ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_policy_with_valid_rules_returns_created_policy() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let name = format!("policy-{}", Uuid::new_v4());
    let payload = json!({
        "name": name,
        "environment": "prod",
        "description": "Test policy",
        "rules": valid_rules()
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200, got {status}\n{raw}");
    assert_eq!(body["success"], true, "{body}");
    assert!(
        body["error"].is_null(),
        "error must be null on success: {body}"
    );

    let data = &body["data"];
    assert!(data.is_object(), "data must be an object: {body}");

    assert_is_uuid(&data["policy_id"], "data.policy_id");
    assert_eq!(data["name"].as_str(), Some(name.as_str()), "{body}");
    assert_eq!(data["environment"].as_str(), Some("prod"), "{body}");
    assert_eq!(data["description"].as_str(), Some("Test policy"), "{body}");
    assert_eq!(data["active_version"].as_i64(), Some(1), "{body}");
    assert!(
        data["created_at"].is_string(),
        "created_at must be a string: {body}"
    );
}

#[tokio::test]
async fn create_policy_without_optional_fields_returns_created_policy() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let payload = json!({
        "name": format!("policy-{}", Uuid::new_v4()),
        "environment": "staging",
        "rules": valid_rules()
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200, got {status}\n{raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = &body["data"];
    assert!(
        data["description"].is_null(),
        "description should be null when omitted: {body}"
    );
    assert_eq!(data["active_version"].as_i64(), Some(1), "{body}");
}

// ── Validation errors ─────────────────────────────────────────────────────────

#[tokio::test]
async fn create_policy_with_empty_name_returns_validation_error() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let payload = json!({
        "name": "",
        "environment": "prod",
        "rules": valid_rules()
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn create_policy_with_empty_environment_returns_validation_error() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let payload = json!({
        "name": format!("policy-{}", Uuid::new_v4()),
        "environment": "",
        "rules": valid_rules()
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

// ── Rules validation errors ───────────────────────────────────────────────────

#[tokio::test]
async fn create_policy_with_empty_rules_array_returns_validation_error() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let payload = json!({
        "name": format!("policy-{}", Uuid::new_v4()),
        "environment": "prod",
        "rules": []
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn create_policy_with_rules_as_object_returns_validation_error() {
    let client = Client::new();
    let token = admin_login(&client).await;

    // rules must be an array — sending an object is invalid
    let payload = json!({
        "name": format!("policy-{}", Uuid::new_v4()),
        "environment": "prod",
        "rules": { "method": "GET", "path": "/users", "roles": ["user"] }
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    let (status, _body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
}

#[tokio::test]
async fn create_policy_with_rule_missing_method_returns_validation_error() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let payload = json!({
        "name": format!("policy-{}", Uuid::new_v4()),
        "environment": "prod",
        "rules": [{ "path": "/users", "roles": ["user"] }]
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    // missing required struct fields fail serde deserialization before the validator runs,
    // so the error code is INVALID_JSON (not VALIDATION_ERROR)
    assert_error_envelope(&body, "INVALID_JSON");
}

#[tokio::test]
async fn create_policy_with_rule_path_not_starting_with_slash_returns_validation_error() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let payload = json!({
        "name": format!("policy-{}", Uuid::new_v4()),
        "environment": "prod",
        "rules": [{ "method": "GET", "path": "users/:id", "roles": ["user"] }]
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn create_policy_with_rule_double_slash_in_path_returns_validation_error() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let payload = json!({
        "name": format!("policy-{}", Uuid::new_v4()),
        "environment": "prod",
        "rules": [{ "method": "GET", "path": "/users//profile", "roles": ["user"] }]
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn create_policy_with_rule_empty_roles_array_returns_validation_error() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let payload = json!({
        "name": format!("policy-{}", Uuid::new_v4()),
        "environment": "prod",
        "rules": [{ "method": "GET", "path": "/users", "roles": [] }]
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn create_policy_with_rule_empty_role_string_returns_validation_error() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let payload = json!({
        "name": format!("policy-{}", Uuid::new_v4()),
        "environment": "prod",
        "rules": [{ "method": "GET", "path": "/users", "roles": [""] }]
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}
