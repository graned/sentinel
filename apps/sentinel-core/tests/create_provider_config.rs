mod common;

use common::{
    auth_assertions::{assert_protected_endpoint, ProtectedExpected, ProtectedTokens},
    helpers::{generate_expired_token, post_json, read_json},
    setup::{get_config_email_url, get_login_user_url, get_register_user_url},
};
use reqwest::{Client, Method};
use serde_json::{json, Value};
use uuid::Uuid;

#[tokio::test]
async fn config_email_is_protected_contract() {
    let client = Client::new();

    let uid = Uuid::new_v4();
    let sid = Uuid::new_v4();
    let expired_token = generate_expired_token(uid, sid).unwrap();

    // whatever you consider "invalid"
    let invalid_token = "this.is.not.a.valid.token";

    let payload = json!({
        "provider": format!("smtp-{}", Uuid::new_v4()),
        "config": {
            "host": "smtp.postmarkapp.com",
            "port": 587,
            "username": "postmark-username",
            "password": "super-secret-password",
            "use_tls": true,
            "from_email": "no-reply@acme.com",
            "from_name": "Acme SaaS"
        },
        "is_active": true
    });

    assert_protected_endpoint(
        &client,
        Method::POST,
        get_config_email_url(),
        Some(payload),
        ProtectedTokens {
            expired: &expired_token,
            invalid: invalid_token,
        },
        ProtectedExpected {
            missing_code: "MISSING_TOKEN",
            expired_code: "EXPIRED_TOKEN",
            invalid_code: "INVALID_TOKEN",
        },
    )
    .await;
}

#[tokio::test]
async fn create_config_email_with_valid_token() {
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

    let payload = json!({
        "provider": format!("smtp-{}", Uuid::new_v4()),
        "config": {
            "host": "smtp.postmarkapp.com",
            "port": 587,
            "username": "postmark-username",
            "password": "super-secret-password",
            "use_tls": true,
            "from_email": "no-reply@acme.com",
            "from_name": "Acme SaaS"
        },
        "is_active": true
    });
    let access_token = body2["data"]["access_token"]
        .as_str()
        .expect("login response must include data.access_token")
        .to_string();

    let res = client
        .post(get_config_email_url())
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.expect("response must be JSON");

    // Success semantics
    assert_eq!(body["success"], true);
    assert!(body["error"].is_null(), "error must be null on success");
    assert!(body["data"].is_object(), "data must be object on success");

    // Data semantics (ProviderConfigResponse)
    let data = &body["data"];
    assert!(
        data["tenant_id"].is_null(),
        "tenant_id should be null in this test"
    ); // adjust if your app sets it
    assert!(data["provider"].is_string(), "provider must be string");
    assert!(data["is_active"].is_boolean(), "is_active must be boolean");

    // if your response provider is normalized (like "smtp") rather than "smtp-<uuid>",
    // don't assert equality to provider_value; instead assert it contains "smtp"
    let provider_res = data["provider"].as_str().unwrap();
    assert!(
        provider_res.contains("smtp"),
        "provider should contain 'smtp', got: {provider_res}"
    );

    // ids
    assert_is_uuid(&data["configuration_id"], "data.configuration_id");

    // redaction object checks
    assert_redacted_object(
        &data["config_redacted"],
        &[
            "host",
            "port",
            "username",
            "password",
            "use_tls",
            "from_email",
            "from_name",
        ],
    );
}

//#######################################################################################
// Helpers
//#######################################################################################
fn assert_is_uuid(v: &Value, path: &str) {
    let s = v
        .as_str()
        .unwrap_or_else(|| panic!("{path} must be a string"));
    Uuid::parse_str(s).unwrap_or_else(|_| panic!("{path} must be a valid UUID, got: {s}"));
}

fn assert_redacted_object(obj: &Value, expected_keys: &[&str]) {
    assert!(obj.is_object(), "config_redacted must be an object");
    for k in expected_keys {
        assert!(obj.get(*k).is_some(), "config_redacted missing key: {k}");
        assert_eq!(
            obj[*k],
            Value::String("****".to_string()),
            "config_redacted.{k} must be redacted to '****'"
        );
    }
}
