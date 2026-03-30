mod common;

use common::{
    helpers::{assert_error_envelope, post_json, read_json},
    setup::get_register_user_url,
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

fn reg_payload(password: &str) -> serde_json::Value {
    json!({
        "first_name": "Test",
        "last_name": "User",
        "email": format!("pwval-{}@example.com", Uuid::new_v4()),
        "password": password
    })
}

#[tokio::test]
async fn register_with_11_char_password_returns_400() {
    let client = Client::new();
    let res = post_json(&client, get_register_user_url(), reg_payload("Abc1!fghijk")).await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 400, "expected 400 for 11-char password: {raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn register_with_all_lowercase_12_char_returns_400() {
    let client = Client::new();
    // 12 chars but no uppercase
    let res = post_json(
        &client,
        get_register_user_url(),
        reg_payload("abc1!fghijkl"),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 400, "expected 400 for no-uppercase password: {raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn register_with_no_digit_returns_400() {
    let client = Client::new();
    // 12 chars, has upper, lower, special but no digit
    let res = post_json(
        &client,
        get_register_user_url(),
        reg_payload("Abcdefgh!@#$"),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 400, "expected 400 for no-digit password: {raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn register_with_no_special_char_returns_400() {
    let client = Client::new();
    // 12 chars, upper, lower, digit, no special
    let res = post_json(
        &client,
        get_register_user_url(),
        reg_payload("Abcdefgh1234"),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 400, "expected 400 for no-special password: {raw}");
    assert_error_envelope(&body, "VALIDATION_ERROR");
}

#[tokio::test]
async fn register_with_strong_password_returns_200() {
    let client = Client::new();
    // 16 chars, upper, lower, digit, special
    let res = post_json(
        &client,
        get_register_user_url(),
        reg_payload("T3stP@ssw0rd#Sec"),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "expected 200 for strong password: {raw}");
}
