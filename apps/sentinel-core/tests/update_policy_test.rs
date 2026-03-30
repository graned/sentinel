mod common;

use common::{
    helpers::{admin_login, assert_error_envelope, read_json},
    setup::{get_create_policy_url, get_update_policy_rules_url},
};
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn valid_rules() -> Value {
    json!([
        { "method": "GET",  "path": "/users/:id", "roles": ["user", "admin"] },
        { "method": "POST", "path": "/users",     "roles": ["admin"] }
    ])
}

fn updated_rules() -> Value {
    json!([
        { "method": "GET",    "path": "/users/:id",    "roles": ["user", "admin"] },
        { "method": "POST",   "path": "/users",        "roles": ["admin"] },
        { "method": "DELETE", "path": "/users/:id",    "roles": ["admin"] }
    ])
}

/// Creates a policy and returns its `policy_id` and the admin token used.
async fn create_policy(client: &Client) -> (Uuid, String) {
    let token = admin_login(client).await;
    let payload = json!({
        "name": format!("policy-{}", Uuid::new_v4()),
        "environment": "prod",
        "rules": valid_rules()
    });

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("create policy request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "policy creation failed: {raw}");

    let id_str = body
        .pointer("/data/policy_id")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing data.policy_id in create response: {body}"));

    let id = Uuid::parse_str(id_str)
        .unwrap_or_else(|_| panic!("data.policy_id is not a valid UUID: {id_str}"));
    (id, token)
}

// ── Success ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn update_policy_rules_returns_next_version() {
    let client = Client::new();
    let (policy_id, token) = create_policy(&client).await;

    let payload = json!({ "rules": updated_rules() });

    let res = client
        .put(get_update_policy_rules_url(policy_id))
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
    assert_eq!(
        data["policy_id"].as_str(),
        Some(policy_id.to_string().as_str()),
        "policy_id must match: {body}"
    );
    assert_eq!(
        data["activated_version"].as_i64(),
        Some(2),
        "first update must activate version 2: {body}"
    );
}

#[tokio::test]
async fn update_policy_rules_increments_version_on_each_update() {
    let client = Client::new();
    let (policy_id, token) = create_policy(&client).await;

    // First update → version 2
    let res = client
        .put(get_update_policy_rules_url(policy_id))
        .bearer_auth(&token)
        .json(&json!({ "rules": updated_rules() }))
        .send()
        .await
        .expect("request failed");
    let (_, body, _) = read_json(res).await;
    assert_eq!(
        body["data"]["activated_version"].as_i64(),
        Some(2),
        "{body}"
    );

    // Second update → version 3
    let res = client
        .put(get_update_policy_rules_url(policy_id))
        .bearer_auth(&token)
        .json(&json!({ "rules": valid_rules() }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200, got {status}\n{raw}");
    assert_eq!(
        body["data"]["activated_version"].as_i64(),
        Some(3),
        "{body}"
    );
}

// ── Not found ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn update_policy_rules_with_nonexistent_policy_id_returns_error() {
    let client = Client::new();
    let token = admin_login(&client).await;
    let nonexistent_id = Uuid::new_v4();

    let res = client
        .put(get_update_policy_rules_url(nonexistent_id))
        .bearer_auth(&token)
        .json(&json!({ "rules": valid_rules() }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(
        status, 404,
        "expected 404 for unknown policy, got {status}\n{raw}"
    );
    assert_error_envelope(&body, "NOT_FOUND");
}

// ── Rules validation errors ───────────────────────────────────────────────────

#[tokio::test]
async fn update_policy_rules_with_empty_rules_array_returns_validation_error() {
    let client = Client::new();
    let (policy_id, token) = create_policy(&client).await;

    let res = client
        .put(get_update_policy_rules_url(policy_id))
        .bearer_auth(&token)
        .json(&json!({ "rules": [] }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn update_policy_rules_with_rules_as_object_returns_error() {
    let client = Client::new();
    let (policy_id, token) = create_policy(&client).await;

    // rules must be an array — sending an object is invalid
    let res = client
        .put(get_update_policy_rules_url(policy_id))
        .bearer_auth(&token)
        .json(&json!({
            "rules": { "method": "GET", "path": "/users", "roles": ["user"] }
        }))
        .send()
        .await
        .expect("request failed");
    let (status, _body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
}

#[tokio::test]
async fn update_policy_rules_with_rule_missing_method_returns_validation_error() {
    let client = Client::new();
    let (policy_id, token) = create_policy(&client).await;

    let res = client
        .put(get_update_policy_rules_url(policy_id))
        .bearer_auth(&token)
        .json(&json!({
            "rules": [{ "path": "/users", "roles": ["user"] }]
        }))
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
async fn update_policy_rules_with_rule_path_not_starting_with_slash_returns_validation_error() {
    let client = Client::new();
    let (policy_id, token) = create_policy(&client).await;

    let res = client
        .put(get_update_policy_rules_url(policy_id))
        .bearer_auth(&token)
        .json(&json!({
            "rules": [{ "method": "GET", "path": "users/:id", "roles": ["user"] }]
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn update_policy_rules_with_rule_empty_roles_array_returns_validation_error() {
    let client = Client::new();
    let (policy_id, token) = create_policy(&client).await;

    let res = client
        .put(get_update_policy_rules_url(policy_id))
        .bearer_auth(&token)
        .json(&json!({
            "rules": [{ "method": "GET", "path": "/users", "roles": [] }]
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn update_policy_rules_with_rule_double_slash_in_path_returns_validation_error() {
    let client = Client::new();
    let (policy_id, token) = create_policy(&client).await;

    let res = client
        .put(get_update_policy_rules_url(policy_id))
        .bearer_auth(&token)
        .json(&json!({
            "rules": [{ "method": "GET", "path": "/users//profile", "roles": ["user"] }]
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 400, "expected 400, got {status}\n{raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}
