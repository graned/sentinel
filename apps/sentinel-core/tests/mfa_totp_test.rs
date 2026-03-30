mod common;

use common::{
    helpers::{assert_error_envelope, post_json, read_json},
    setup::{
        get_login_user_url, get_mfa_totp_confirm_url, get_mfa_totp_start_url, get_mfa_verify_url,
        get_register_user_url,
    },
};
use reqwest::Client;
use serde_json::{json, Value};
use totp_rs::TOTP;
use uuid::Uuid;

/// Register a user and return their credentials.
async fn register_user(client: &Client) -> (String, String) {
    let email = format!("mfa-user-{}@example.com", Uuid::new_v4());
    let password = "T3stP@ssw0rd#Sec";

    let payload = json!({
        "first_name": "MFA",
        "last_name": "Tester",
        "email": email,
        "avatar_url": null,
        "password": password
    });

    let res = post_json(client, get_register_user_url(), payload).await;
    let (status, body, raw) = read_json(res).await;
    assert!(status == 200, "register failed: {raw}\n{body}");

    (email, password.to_string())
}

/// Login and return the Bearer access token (for non-MFA users).
async fn login_get_token(client: &Client, email: &str, password: &str) -> String {
    let payload = json!({ "email": email, "password": password });
    let res = post_json(client, get_login_user_url(), payload).await;
    let (status, body, raw) = read_json(res).await;
    assert!(status == 200, "login failed: {raw}\n{body}");

    body.pointer("/data/access_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing access_token: {body}"))
        .to_string()
}

/// Start TOTP enrollment (authenticated) and return the otpauth URI.
async fn start_enrollment(client: &Client, bearer: &str) -> String {
    let res = client
        .post(get_mfa_totp_start_url())
        .bearer_auth(bearer)
        .send()
        .await
        .expect("HTTP request failed");
    let (status, body, raw) = read_json(res).await;
    assert!(status == 200, "start_enrollment failed: {raw}\n{body}");

    body.pointer("/data/otpauth_uri")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing otpauth_uri: {body}"))
        .to_string()
}

/// Generate the current TOTP code from an otpauth URI.
fn generate_totp_code(otpauth_uri: &str) -> String {
    let totp = TOTP::from_url(otpauth_uri).expect("Invalid otpauth URI");
    totp.generate_current().expect("TOTP time error")
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn start_enrollment_requires_auth() {
    let client = Client::new();

    let res = client
        .post(get_mfa_totp_start_url())
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(res.status().as_u16(), 401, "expected 401 without Bearer");
}

#[tokio::test]
async fn start_enrollment_returns_otpauth_uri() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login_get_token(&client, &email, &password).await;

    let otpauth_uri = start_enrollment(&client, &token).await;

    assert!(
        otpauth_uri.starts_with("otpauth://totp/"),
        "expected otpauth URI, got: {otpauth_uri}"
    );
}

#[tokio::test]
async fn confirm_enrollment_with_invalid_code_returns_401() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login_get_token(&client, &email, &password).await;

    start_enrollment(&client, &token).await;

    let res = client
        .post(get_mfa_totp_confirm_url())
        .bearer_auth(&token)
        .json(&json!({ "code": "000000" }))
        .send()
        .await
        .expect("HTTP request failed");

    let (status, body, raw) = read_json(res).await;
    assert!(
        status == 401,
        "expected 401 for wrong TOTP code, got {status}\n{raw}"
    );
    assert_error_envelope(&body, "INVALID_MFA_CODE");
}

#[tokio::test]
async fn login_without_mfa_returns_tokens_directly() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;

    let payload = json!({ "email": email, "password": password });
    let res = post_json(&client, get_login_user_url(), payload).await;
    let (status, body, raw) = read_json(res).await;

    assert!(status == 200, "expected 200, got {status}\n{raw}");
    assert_eq!(body.pointer("/success").and_then(|v| v.as_bool()), Some(true));

    let data = &body["data"];
    assert!(
        data.get("access_token").is_some(),
        "expected access_token in response: {body}"
    );
    assert!(
        data.get("refresh_token").is_some(),
        "expected refresh_token in response: {body}"
    );
    assert!(
        data.get("mfa_required").is_none() || data["mfa_required"].as_bool() == Some(false),
        "non-MFA user should not get mfa_required flag: {body}"
    );
}

#[tokio::test]
async fn login_with_mfa_enrolled_returns_challenge() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login_get_token(&client, &email, &password).await;

    // Enroll TOTP
    let uri = start_enrollment(&client, &token).await;
    let code = generate_totp_code(&uri);

    let res = client
        .post(get_mfa_totp_confirm_url())
        .bearer_auth(&token)
        .json(&json!({ "code": code }))
        .send()
        .await
        .expect("HTTP request failed");
    let (status, body, raw) = read_json(res).await;
    assert!(
        status == 200,
        "confirm_enrollment failed: {status}\n{raw}\n{body}"
    );
    assert!(
        body.pointer("/data/recovery_codes").is_some(),
        "expected recovery_codes: {body}"
    );

    // Now re-login: should get MFA challenge
    let payload = json!({ "email": email, "password": password });
    let res = post_json(&client, get_login_user_url(), payload).await;
    let (status, body, raw) = read_json(res).await;

    assert!(status == 200, "expected 200, got {status}\n{raw}");
    let data = &body["data"];
    assert_eq!(
        data["mfa_required"].as_bool(),
        Some(true),
        "expected mfa_required=true: {body}"
    );
    assert!(
        data.get("mfa_session_token").and_then(|v| v.as_str()).is_some(),
        "expected mfa_session_token: {body}"
    );
    assert!(
        data.get("access_token").is_none(),
        "should NOT have access_token in challenge: {body}"
    );
}

#[tokio::test]
async fn verify_mfa_with_invalid_code_returns_401() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login_get_token(&client, &email, &password).await;

    // Enroll TOTP
    let uri = start_enrollment(&client, &token).await;
    let code = generate_totp_code(&uri);
    client
        .post(get_mfa_totp_confirm_url())
        .bearer_auth(&token)
        .json(&json!({ "code": code }))
        .send()
        .await
        .expect("HTTP request failed");

    // Re-login to get MFA challenge
    let payload = json!({ "email": email, "password": password });
    let res = post_json(&client, get_login_user_url(), payload).await;
    let (_, body, _) = read_json(res).await;
    let mfa_token = body
        .pointer("/data/mfa_session_token")
        .and_then(|v| v.as_str())
        .expect("no mfa_session_token")
        .to_string();

    // Verify with a wrong code
    let res = post_json(
        &client,
        get_mfa_verify_url(),
        json!({ "mfa_session_token": mfa_token, "code": "000000" }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert!(
        status == 401,
        "expected 401 for invalid code, got {status}\n{raw}"
    );
    assert_error_envelope(&body, "INVALID_MFA_CODE");
}

#[tokio::test]
async fn verify_mfa_with_recovery_code_works_once() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;
    let token = login_get_token(&client, &email, &password).await;

    // Enroll TOTP and capture recovery codes
    let uri = start_enrollment(&client, &token).await;
    let code = generate_totp_code(&uri);

    let res = client
        .post(get_mfa_totp_confirm_url())
        .bearer_auth(&token)
        .json(&json!({ "code": code }))
        .send()
        .await
        .expect("HTTP request failed");
    let (_, body, _) = read_json(res).await;
    let recovery_codes: Vec<String> = body
        .pointer("/data/recovery_codes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    assert!(!recovery_codes.is_empty(), "no recovery codes returned");
    let recovery_code = recovery_codes[0].clone();

    // Re-login to get MFA challenge
    let payload = json!({ "email": email, "password": password });
    let res = post_json(&client, get_login_user_url(), payload).await;
    let (_, login_body, _) = read_json(res).await;
    let mfa_token = login_body
        .pointer("/data/mfa_session_token")
        .and_then(|v| v.as_str())
        .expect("no mfa_session_token")
        .to_string();

    // Use recovery code — should succeed
    let res = post_json(
        &client,
        get_mfa_verify_url(),
        json!({ "mfa_session_token": mfa_token, "code": recovery_code }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert!(
        status == 200,
        "recovery code should work: {status}\n{raw}"
    );
    assert!(
        body.pointer("/data/access_token").is_some(),
        "expected access_token after MFA verify: {body}"
    );

    // Re-login again to get a fresh challenge
    let payload = json!({ "email": email, "password": password });
    let res = post_json(&client, get_login_user_url(), payload).await;
    let (_, login_body2, _) = read_json(res).await;
    let mfa_token2 = login_body2
        .pointer("/data/mfa_session_token")
        .and_then(|v| v.as_str())
        .expect("no mfa_session_token on second login")
        .to_string();

    // Try same recovery code again — should fail
    let res = post_json(
        &client,
        get_mfa_verify_url(),
        json!({ "mfa_session_token": mfa_token2, "code": recovery_code }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert!(
        status == 401,
        "used recovery code should be rejected: {status}\n{raw}"
    );
    assert_error_envelope(&body, "INVALID_MFA_CODE");
}

/// Submit 5 wrong MFA codes, then the 6th attempt must return 429 MFA_ATTEMPT_LIMIT_EXCEEDED.
#[tokio::test]
async fn mfa_verify_attempt_limit_returns_429_on_6th_try() {
    let client = Client::new();
    let (email, password) = register_user(&client).await;

    // Enroll MFA
    let token = login_get_token(&client, &email, &password).await;
    let uri = start_enrollment(&client, &token).await;

    let code = generate_totp_code(&uri);
    client
        .post(get_mfa_totp_confirm_url())
        .bearer_auth(&token)
        .json(&json!({ "code": code }))
        .send()
        .await
        .expect("confirm enrollment request failed");

    // Get MFA challenge token
    let res = post_json(
        &client,
        get_login_user_url(),
        json!({ "email": email, "password": password }),
    )
    .await;
    let (_, login_body, raw) = read_json(res).await;
    let mfa_token = login_body
        .pointer("/data/mfa_session_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("no mfa_session_token: {raw}"))
        .to_string();

    // Submit 5 wrong codes — all should fail with 401
    for i in 0..5 {
        let res = post_json(
            &client,
            get_mfa_verify_url(),
            json!({ "mfa_session_token": mfa_token, "code": "000000" }),
        )
        .await;
        let (status, body, raw) = read_json(res).await;
        assert_eq!(
            status, 401,
            "attempt {i}: expected 401 for wrong code: {raw}"
        );
        assert_error_envelope(&body, "INVALID_MFA_CODE");
    }

    // 6th attempt — should be rate-limited
    let res = post_json(
        &client,
        get_mfa_verify_url(),
        json!({ "mfa_session_token": mfa_token, "code": "000000" }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(
        status, 429,
        "6th attempt should be rate-limited: {raw}"
    );
    assert_error_envelope(&body, "MFA_ATTEMPT_LIMIT_EXCEEDED");
}
