use chrono::{Duration, Utc};
use reqwest::Client;
use rusty_paseto::prelude::*;
use serde_json::Value;
use std::env;
use uuid::Uuid;

use super::setup::get_login_user_url;

/// A strong password that passes the registration password policy.
/// Meets all requirements: ≥12 chars, upper, lower, digit, special character.
#[allow(dead_code)]
pub const TEST_PASSWORD: &str = "T3stP@ssw0rd#Sec";

/// Generate a unique fake IP per call so integration tests never share
/// a rate-limiter bucket with each other. Rate-limit tests use their own
/// helper (`post_with_ip`) and are unaffected by this.
fn unique_ip() -> String {
    let id = Uuid::new_v4().to_string().replace('-', "");
    let a = u8::from_str_radix(&id[0..2], 16).unwrap_or(1);
    let b = u8::from_str_radix(&id[2..4], 16).unwrap_or(1);
    let c = u8::from_str_radix(&id[4..6], 16).unwrap_or(1);
    format!("10.{a}.{b}.{c}")
}

/// Helper to POST JSON payloads
#[allow(dead_code)]
pub async fn post_json(client: &Client, url: String, payload: Value) -> reqwest::Response {
    client
        .post(url)
        .header("X-Forwarded-For", unique_ip())
        .json(&payload)
        .send()
        .await
        .expect("HTTP request failed (server unreachable?)")
}

/// Helper to PUT JSON payloads
#[allow(dead_code)]
pub async fn put_json(client: &Client, url: String, payload: Value) -> reqwest::Response {
    client
        .put(url)
        .header("X-Forwarded-For", unique_ip())
        .json(&payload)
        .send()
        .await
        .expect("HTTP request failed (server unreachable?)")
}

/// Helper to read and parse JSON responses
#[allow(dead_code)]
pub async fn read_json(res: reqwest::Response) -> (u16, Value, String) {
    let status = res.status().as_u16();
    let text = res.text().await.unwrap_or_default();
    let json: Value = serde_json::from_str(&text).unwrap_or_else(|e| {
        panic!("response was not JSON: {e}\nstatus={status}\nbody:\n{text}");
    });
    (status, json, text)
}

/// Shared assertions for error envelope
#[allow(dead_code)]
pub fn assert_error_envelope<'a>(body: &'a Value, expected_code: &'a str) -> &'a str {
    assert_eq!(
        body.get("success").and_then(|v| v.as_bool()),
        Some(false),
        "{body}"
    );

    assert!(
        body.get("data").map(|v| v.is_null()).unwrap_or(false),
        "expected data=null: {body}"
    );

    let code = body
        .pointer("/error/code")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing error.code: {body}"));

    assert_eq!(code, expected_code, "unexpected error.code: {body}");

    let msg = body
        .pointer("/error/message")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing error.message: {body}"));

    assert!(body.get("timestamp").is_some(), "missing timestamp: {body}");
    assert!(
        body.get("request_id").is_some(),
        "missing request_id: {body}"
    );

    msg
}

/// Shared assertions for successful login responses
#[allow(dead_code)]
pub fn assert_login_success_envelope(body: &Value) {
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
/// Disable TOTP MFA for the seeded admin user so login returns an access_token
/// directly (rather than an mfa_session_token challenge).
#[allow(dead_code)]
pub async fn disable_admin_mfa() {
    use dotenvy::dotenv;
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set for test");
    let (client, connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
        .await
        .expect("DB connection failed");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("DB connection error: {e}");
        }
    });
    client
        .execute(
            "UPDATE user_mfa_totp SET enabled = false \
             WHERE user_id = (SELECT user_id FROM user_identities WHERE email = 'admin@sentinel.local')",
            &[],
        )
        .await
        .expect("DB update user_mfa_totp failed");
}

/// Login as the seeded super-admin and return a Bearer access token.
/// Disables MFA first so that the login returns a token directly.
#[allow(dead_code)]
pub async fn admin_login(client: &Client) -> String {
    disable_admin_mfa().await;
    let res = post_json(
        client,
        get_login_user_url(),
        serde_json::json!({ "email": "admin@sentinel.local", "password": "admin" }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "admin login failed: {raw}");
    body.pointer("/data/access_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing access_token in admin login: {body}"))
        .to_string()
}

/// Generate expired token
#[allow(dead_code)]
pub fn generate_expired_token(
    user_id: Uuid,
    session_id: Uuid,
) -> Result<String, Box<dyn std::error::Error>> {
    let hex_key = env::var("HEX_KEY").expect("Hex key is missing");
    let hex_bytes = hex::decode(hex_key).expect("Invalid hex key");
    let session_enc_key: [u8; 32] = hex_bytes
        .try_into()
        .expect("Session enc key must be 32 bytes");

    let key = PasetoSymmetricKey::<V4, Local>::from(Key::from(&session_enc_key));

    let exp_rfc3339 = (Utc::now() - Duration::minutes(1)).to_rfc3339();

    let sub = "Sentinel Token".to_string();
    let token = PasetoBuilder::<V4, Local>::default()
        // set reserved claims
        .set_claim(SubjectClaim::from(sub.as_str()))
        .set_claim(CustomClaim::try_from(("sid", session_id.to_string()))?)
        .set_claim(CustomClaim::try_from(("uid", user_id.to_string()))?)
        .set_claim(ExpirationClaim::try_from(exp_rfc3339.as_str())?)
        .build(&key)
        .expect("failed to build token");
    Ok(token)
}
