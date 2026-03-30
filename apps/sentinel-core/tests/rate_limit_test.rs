mod common;

use common::setup::{get_forgot_password_url, get_login_user_url, get_register_user_url};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

/// Generate a unique fake IP for this test run to avoid rate-limiter state contamination
/// across test invocations. Uses the first 4 hex chars of a UUID mapped to 10.x.x.x.
fn unique_test_ip() -> String {
    let id = Uuid::new_v4().to_string().replace('-', "");
    let a = u8::from_str_radix(&id[0..2], 16).unwrap_or(1);
    let b = u8::from_str_radix(&id[2..4], 16).unwrap_or(1);
    let c = u8::from_str_radix(&id[4..6], 16).unwrap_or(1);
    format!("10.{a}.{b}.{c}")
}

/// POST JSON with a spoofed X-Forwarded-For header so the rate limiter uses a unique IP.
async fn post_with_ip(client: &Client, url: &str, body: serde_json::Value, ip: &str) -> reqwest::Response {
    client
        .post(url)
        .header("X-Forwarded-For", ip)
        .json(&body)
        .send()
        .await
        .expect("HTTP request failed")
}

/// Submit 6 login requests from the same unique test IP.
/// The 6th request must return 429.
#[tokio::test]
async fn login_rate_limit_returns_429_after_5_requests() {
    let client = Client::new();
    let ip = unique_test_ip();
    let email = format!("rl-login-{}@example.com", Uuid::new_v4());

    // Register user first (uses a different route / rate limiter, unique IP)
    client
        .post(get_register_user_url())
        .header("X-Forwarded-For", &ip)
        .json(&json!({
            "first_name": "Rate",
            "last_name": "Limit",
            "email": email,
            "password": "T3stP@ssw0rd#Sec"
        }))
        .send()
        .await
        .expect("request failed");

    // Fire 5 login requests — each should pass the rate limiter
    for i in 0..5 {
        let res = post_with_ip(
            &client,
            &get_login_user_url(),
            json!({ "email": email, "password": "wrong-but-valid-len" }),
            &ip,
        )
        .await;
        let status = res.status().as_u16();
        assert_ne!(
            status, 429,
            "attempt {i}: unexpected rate limit, expected not-429 got {status}"
        );
    }

    // 6th request must be rate-limited
    let res = post_with_ip(
        &client,
        &get_login_user_url(),
        json!({ "email": email, "password": "wrong-but-valid-len" }),
        &ip,
    )
    .await;
    let status = res.status().as_u16();
    assert_eq!(status, 429, "6th login attempt should be rate-limited");
}

/// Submit 11 forgot_password requests. The 11th must return 429.
#[tokio::test]
async fn forgot_password_rate_limit_returns_429_after_10_requests() {
    let client = Client::new();
    let ip = unique_test_ip();
    let email = format!("rl-forgot-{}@example.com", Uuid::new_v4());

    // Fire 10 forgot_password requests
    for i in 0..10 {
        let res = post_with_ip(
            &client,
            &get_forgot_password_url(),
            json!({ "email": email }),
            &ip,
        )
        .await;
        let status = res.status().as_u16();
        assert_ne!(
            status, 429,
            "attempt {i}: unexpected rate limit, expected not-429 got {status}"
        );
    }

    // 11th request must be rate-limited
    let res = post_with_ip(
        &client,
        &get_forgot_password_url(),
        json!({ "email": email }),
        &ip,
    )
    .await;
    let status = res.status().as_u16();
    assert_eq!(status, 429, "11th forgot_password attempt should be rate-limited");
}
