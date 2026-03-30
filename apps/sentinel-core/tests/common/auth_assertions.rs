use reqwest::{Client, Method};
use serde_json::Value;

/// Contract for all protected endpoints:
/// - missing token => 401 + code
/// - expired token => 401 + code
/// - invalid token => 401 + code
#[allow(dead_code)]
pub async fn assert_protected_endpoint(
    client: &Client,
    method: Method,
    url: String,
    payload: Option<Value>,
    tokens: ProtectedTokens<'_>,
    expected: ProtectedExpected<'_>,
) {
    // 1) Missing token
    let res = send(client, method.clone(), &url, payload.clone(), None).await;
    assert_eq!(
        res.status().as_u16(),
        401,
        "missing token should be 401 for {method} {url}"
    );
    assert_error_code(res, expected.missing_code, "missing token").await;

    // 2) Expired token
    let res = send(
        client,
        method.clone(),
        &url,
        payload.clone(),
        Some(tokens.expired),
    )
    .await;
    assert_eq!(
        res.status().as_u16(),
        401,
        "expired token should be 401 for {method} {url}"
    );
    assert_error_code(res, expected.expired_code, "expired token").await;

    // 3) Invalid token
    let res = send(client, method, &url, payload, Some(tokens.invalid)).await;
    assert_eq!(
        res.status().as_u16(),
        401,
        "invalid token should be 401 for {url}"
    );
    assert_error_code(res, expected.invalid_code, "invalid token").await;
}

#[allow(dead_code)]
pub struct ProtectedTokens<'a> {
    pub expired: &'a str,
    pub invalid: &'a str,
}

#[allow(dead_code)]
pub struct ProtectedExpected<'a> {
    pub missing_code: &'a str,
    pub expired_code: &'a str,
    pub invalid_code: &'a str,
}

#[allow(dead_code)]
async fn send(
    client: &Client,
    method: Method,
    url: &str,
    payload: Option<Value>,
    bearer: Option<&str>,
) -> reqwest::Response {
    let mut req = client.request(method, url);

    if let Some(token) = bearer {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    if let Some(p) = payload {
        req = req.json(&p);
    }

    req.send().await.expect("request failed")
}

#[allow(dead_code)]
async fn assert_error_code(res: reqwest::Response, expected_code: &str, label: &str) {
    let body: Value = res.json().await.expect("response must be JSON");

    assert_eq!(body["success"], false, "{label}: success must be false");
    assert!(body["data"].is_null(), "{label}: data should be null");
    assert!(body["error"].is_object(), "{label}: error must be object");

    assert_eq!(
        body["error"]["code"], expected_code,
        "{label}: wrong error code"
    );
}
