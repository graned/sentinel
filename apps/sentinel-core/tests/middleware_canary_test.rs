mod common;

use common::{
    helpers::{
        admin_login, assert_error_envelope, assert_login_success_envelope, post_json, read_json,
    },
    setup::{
        get_admin_policies_url, get_admin_policy_url, get_authenticate_token_url,
        get_create_policy_url, get_login_user_url, get_register_user_url,
        get_update_policy_rules_url, get_user_canary_url,
    },
};
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────────────

struct UserSession {
    token: String,
    roles: Vec<String>,
}

/// Directly mark a user's email as verified in the DB via tokio-postgres.
/// Required because the authorize_middleware blocks unverified users before policy evaluation.
async fn mark_email_verified(email: &str) {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set for test");
    let (client, connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
        .await
        .expect("DB connection failed");

    // Drive the connection in the background
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

/// Register a fresh user, pre-verify their email in the DB, login, and
/// authenticate the token to retrieve roles.
/// Email pre-verification is required because `authorize_middleware` blocks
/// unverified users before policy evaluation.
async fn register_and_login(client: &Client) -> UserSession {
    let email = format!("canary-{}@test.com", Uuid::new_v4());
    let password = "T3stP@ssw0rd#Sec";

    let res = post_json(
        client,
        get_register_user_url(),
        json!({
            "first_name": "Sentinel",
            "last_name": "Canary",
            "email": email,
            "password": password
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "registration failed: {raw}");

    // Pre-verify the email so authorize_middleware doesn't block the test user.
    mark_email_verified(&email).await;

    let res = post_json(
        client,
        get_login_user_url(),
        json!({ "email": email, "password": password }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");
    assert_login_success_envelope(&body);
    let token = body["data"]["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("no access_token: {raw}"))
        .to_string();

    // Authenticate token to discover the user's actual roles
    let res = client
        .post(get_authenticate_token_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("authenticate request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "token authentication failed: {raw}");

    let roles: Vec<String> = body["data"]["roles"]
        .as_array()
        .unwrap_or_else(|| panic!("missing roles array: {raw}"))
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    assert!(
        !roles.is_empty(),
        "user has no roles — cannot create a meaningful policy"
    );

    UserSession { token, roles }
}

/// Delete all policies whose name starts with the given prefix.
/// Used to remove leftover test policies from previous runs so OR-semantics
/// evaluation doesn't grant access unexpectedly.
async fn cleanup_policies_with_prefix(client: &Client, admin_token: &str, prefix: &str) {
    let res = client
        .get(get_admin_policies_url())
        .bearer_auth(admin_token)
        .send()
        .await
        .expect("list policies request failed");
    let (_, body, _) = read_json(res).await;
    if let Some(policies) = body["data"].as_array() {
        for policy in policies {
            let name = policy["name"].as_str().unwrap_or("");
            if name.starts_with(prefix) {
                if let Some(id) = policy["policy_id"].as_str() {
                    let _ = client
                        .delete(get_admin_policy_url(id.parse().expect("invalid policy_id")))
                        .bearer_auth(admin_token)
                        .send()
                        .await;
                }
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Verify that no bearer token → 401. No policy interaction, always safe to run in parallel.
#[tokio::test]
async fn canary_returns_401_without_token() {
    let client = Client::new();

    let res = client
        .get(get_user_canary_url())
        .send()
        .await
        .expect("canary request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401 without token: {raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

/// Full middleware smoke test executed as a single sequential scenario to avoid
/// races over the shared engine cache:
///   1. Deny  — policy exists but the canary path is not in the rules  → 403
///   2. Allow — rules updated to include the canary path               → 200
#[tokio::test]
async fn canary_deny_then_allow_after_policy_update() {
    let client = Client::new();
    let session = register_and_login(&client).await;
    let admin_token = admin_login(&client).await;

    // Remove leftover "canary-smoke-*" policies from previous test runs so
    // OR-semantics evaluation doesn't grant access in the deny step.
    cleanup_policies_with_prefix(&client, &admin_token, "canary-smoke-").await;

    // ── Step 1: create a policy that does NOT cover the canary path ──────────

    let deny_rules: Vec<serde_json::Value> = session
        .roles
        .iter()
        .map(|role| {
            json!({
                "method": "GET",
                "path": "/v1/api/other/endpoint",
                "roles": [role]
            })
        })
        .collect();

    let res = client
        .post(get_create_policy_url())
        .bearer_auth(&admin_token)
        .json(&json!({
            "name": format!("canary-smoke-{}", Uuid::new_v4()),
            "environment": "prod",
            "rules": deny_rules
        }))
        .send()
        .await
        .expect("policy creation request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "policy creation failed: {raw}");
    let policy_id: Uuid = body["data"]["policy_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("missing policy_id: {raw}"));

    // Canary path is not covered → access denied
    let res = client
        .get(get_user_canary_url())
        .bearer_auth(&session.token)
        .send()
        .await
        .expect("canary request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 403, "expected 403 before granting access: {raw}");
    assert_error_envelope(&body, "FORBIDDEN");

    // ── Step 2: update the policy to grant access to the canary path ─────────

    let allow_rules: Vec<serde_json::Value> = session
        .roles
        .iter()
        .map(|role| {
            json!({
                "method": "GET",
                "path": "/v1/api/user/canary",
                "roles": [role]
            })
        })
        .collect();

    let res = client
        .put(get_update_policy_rules_url(policy_id))
        .bearer_auth(&admin_token)
        .json(&json!({ "rules": allow_rules }))
        .send()
        .await
        .expect("policy update request failed");
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "policy update failed: {raw}");

    // Now the canary should return the funny message
    let res = client
        .get(get_user_canary_url())
        .bearer_auth(&session.token)
        .send()
        .await
        .expect("canary request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200 after granting access: {raw}");
    assert_eq!(body["success"], true, "{body}");
    let msg = body["data"].as_str().unwrap_or_default();
    assert!(
        !msg.is_empty(),
        "expected a funny message, got nothing: {body}"
    );
    assert!(
        msg.contains(&session.roles[0]),
        "message should mention the user's role: {msg}"
    );

    // Clean up the test policy so it doesn't pollute future runs under OR semantics.
    let _ = client
        .delete(get_admin_policy_url(policy_id))
        .bearer_auth(&admin_token)
        .send()
        .await;
}
