mod common;

use common::{
    helpers::{assert_error_envelope, post_json, read_json, TEST_PASSWORD},
    setup::{
        get_config_email_item_url, get_config_email_reveal_url, get_config_email_url,
        get_login_user_url, get_register_user_url,
    },
};
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn smtp_config() -> Value {
    json!({
        "host": "smtp.example.com",
        "port": 587,
        "username": "user@example.com",
        "password": "secret-pass",
        "from_email": "no-reply@example.com"
    })
}

async fn register_fresh_user(client: &Client) -> (String, String, Uuid) {
    let email = format!("provider-cfg-{}@example.com", Uuid::new_v4());
    let payload = json!({
        "first_name": "Config",
        "last_name":  "Tester",
        "email":      email,
        "password":   TEST_PASSWORD
    });
    let res = post_json(client, get_register_user_url(), payload).await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "register failed: {raw}");
    let user_id: Uuid = body
        .pointer("/data/user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .expect("missing user_id in register response");
    (email, TEST_PASSWORD.to_string(), user_id)
}

async fn mark_email_verified(email: &str) {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let (client, conn) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
        .await
        .expect("DB connection failed");
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("DB connection error: {e}");
        }
    });
    client
        .execute(
            "UPDATE user_identities SET email_verified = true WHERE email = $1",
            &[&email],
        )
        .await
        .expect("mark_email_verified failed");
}

async fn grant_admin_role(user_id: Uuid) {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let (client, conn) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
        .await
        .expect("DB connection failed");
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("DB connection error: {e}");
        }
    });

    // Find or create an admin role — embed UUID literals directly to avoid
    // the tokio-postgres UUID trait requirement (no with-uuid-1 feature).
    let row = client
        .query_opt(
            "SELECT role_id::TEXT FROM roles WHERE \"type\" = 'admin' LIMIT 1",
            &[],
        )
        .await
        .expect("query roles failed");

    let role_id_str: String = if let Some(r) = row {
        r.get(0)
    } else {
        let new_id = Uuid::new_v4();
        client
            .execute(
                &format!(
                    "INSERT INTO roles (role_id, \"type\", name, description, created_at, updated_at) \
                     VALUES ('{new_id}', 'admin', 'Admin', 'Admin role', NOW(), NOW())"
                ),
                &[],
            )
            .await
            .expect("insert role failed");
        new_id.to_string()
    };

    // Insert user_role — embed UUIDs as literals
    let user_role_id = Uuid::new_v4();
    client
        .execute(
            &format!(
                "INSERT INTO user_roles (user_role_id, user_id, role_id, created_at) \
                 VALUES ('{user_role_id}', '{user_id}', '{role_id_str}', NOW()) ON CONFLICT DO NOTHING"
            ),
            &[],
        )
        .await
        .expect("insert user_role failed");
}

async fn login(client: &Client, email: &str, password: &str) -> String {
    let res = post_json(
        client,
        get_login_user_url(),
        json!({ "email": email, "password": password }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");
    body.pointer("/data/access_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing access_token: {body}"))
        .to_string()
}

/// Register a user, verify email, grant admin, return bearer token.
async fn setup_admin_user(client: &Client) -> String {
    let (email, password, user_id) = register_fresh_user(client).await;
    mark_email_verified(&email).await;
    grant_admin_role(user_id).await;
    login(client, &email, &password).await
}

/// Register a regular (non-admin) user and return bearer token.
async fn setup_regular_user(client: &Client) -> String {
    let (email, password, _) = register_fresh_user(client).await;
    mark_email_verified(&email).await;
    login(client, &email, &password).await
}

/// Create a provider config via API, return the config_id from the response.
async fn create_config(client: &Client, token: &str) -> Uuid {
    let res = client
        .post(get_config_email_url())
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "provider": format!("smtp-{}", Uuid::new_v4()),
            "config": smtp_config(),
            "is_active": true
        }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "create config failed: {raw}");
    body.pointer("/data/configuration_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .expect("missing configuration_id in create response")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_provider_configs_unauthenticated() {
    let client = Client::new();
    let res = client
        .get(get_config_email_url())
        .send()
        .await
        .expect("request failed");
    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 401);
    assert_error_envelope(&body, "MISSING_TOKEN");
}

#[tokio::test]
async fn test_list_provider_configs_non_admin_returns_403() {
    let client = Client::new();
    let token = setup_regular_user(&client).await;
    let res = client
        .get(get_config_email_url())
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("request failed");
    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 403);
    assert_error_envelope(&body, "FORBIDDEN");
}

#[tokio::test]
async fn test_create_and_list_provider_config() {
    let client = Client::new();
    let token = setup_admin_user(&client).await;

    // Create a config
    let config_id = create_config(&client, &token).await;

    // List should include it
    let res = client
        .get(get_config_email_url())
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "list failed: {raw}");
    assert_eq!(body["success"], true);

    let items = body["data"].as_array().expect("data must be array");
    let found = items
        .iter()
        .any(|item| item["configuration_id"].as_str() == Some(&config_id.to_string()));
    assert!(found, "created config not found in list");

    // Verify redaction
    if let Some(item) = items
        .iter()
        .find(|item| item["configuration_id"].as_str() == Some(&config_id.to_string()))
    {
        let redacted = &item["config_redacted"];
        assert!(redacted.is_object(), "config_redacted must be object");
        for (_, v) in redacted.as_object().unwrap() {
            assert_eq!(
                v.as_str(),
                Some("****"),
                "all config values must be redacted to '****'"
            );
        }
    }
}

#[tokio::test]
async fn test_update_provider_config() {
    let client = Client::new();
    let token = setup_admin_user(&client).await;
    let config_id = create_config(&client, &token).await;

    let updated_config = json!({
        "host": "smtp-updated.example.com",
        "port": 465,
        "username": "updated@example.com",
        "password": "new-secret"
    });

    let res = client
        .put(get_config_email_item_url(config_id))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "config": updated_config, "is_active": false }))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "update failed: {raw}");
    assert_eq!(body["success"], true);

    let data = &body["data"];
    assert_eq!(
        data["configuration_id"].as_str(),
        Some(config_id.to_string().as_str())
    );
    assert_eq!(data["is_active"].as_bool(), Some(false));

    // Values must still be redacted
    let redacted = &data["config_redacted"];
    assert!(redacted.is_object());
    for (_, v) in redacted.as_object().unwrap() {
        assert_eq!(v.as_str(), Some("****"));
    }
}

#[tokio::test]
async fn test_delete_provider_config() {
    let client = Client::new();
    let token = setup_admin_user(&client).await;
    let config_id = create_config(&client, &token).await;

    // Delete it
    let res = client
        .delete(get_config_email_item_url(config_id))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("request failed");
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "delete failed: {raw}");

    // Should no longer appear in list
    let res = client
        .get(get_config_email_url())
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("request failed");
    let (_, body, _) = read_json(res).await;
    let items = body["data"].as_array().expect("data must be array");
    let still_present = items
        .iter()
        .any(|item| item["configuration_id"].as_str() == Some(&config_id.to_string()));
    assert!(!still_present, "deleted config should not appear in list");
}

#[tokio::test]
async fn test_delete_nonexistent_provider_config() {
    let client = Client::new();
    let token = setup_admin_user(&client).await;
    let fake_id = Uuid::new_v4();

    let res = client
        .delete(get_config_email_item_url(fake_id))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("request failed");
    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 404);
    assert_error_envelope(&body, "NOT_FOUND");
}

#[tokio::test]
async fn test_reveal_provider_config() {
    let client = Client::new();
    let token = setup_admin_user(&client).await;
    let config_id = create_config(&client, &token).await;

    let res = client
        .get(get_config_email_reveal_url(config_id))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "reveal failed: {raw}");
    assert_eq!(body["success"], true);

    let data = &body["data"];
    assert_eq!(
        data["configuration_id"].as_str(),
        Some(config_id.to_string().as_str())
    );

    // Decrypted values must NOT be redacted
    let config = &data["config"];
    assert!(config.is_object(), "config must be object");
    let config_obj = config.as_object().unwrap();
    for (_, v) in config_obj {
        assert_ne!(
            v.as_str(),
            Some("****"),
            "revealed config must not contain redacted values"
        );
    }

    // Verify actual values from smtp_config()
    assert_eq!(
        config["host"].as_str(),
        Some("smtp.example.com"),
        "host must match original"
    );
    assert_eq!(
        config["password"].as_str(),
        Some("secret-pass"),
        "password must match original"
    );
}

#[tokio::test]
async fn test_reveal_unauthenticated() {
    let client = Client::new();
    let fake_id = Uuid::new_v4();
    let res = client
        .get(get_config_email_reveal_url(fake_id))
        .send()
        .await
        .expect("request failed");
    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 401);
    assert_error_envelope(&body, "MISSING_TOKEN");
}

#[tokio::test]
async fn test_reveal_non_admin_returns_403() {
    let client = Client::new();
    let token = setup_regular_user(&client).await;
    let fake_id = Uuid::new_v4();

    let res = client
        .get(get_config_email_reveal_url(fake_id))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("request failed");
    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 403);
    assert_error_envelope(&body, "FORBIDDEN");
}
