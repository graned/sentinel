mod common;

use common::assertions::assert_api_envelope_shape;
use common::setup::get_server_url;
use reqwest::Client;
use serde_json::Value;

#[tokio::test]
async fn health_endpoint_preserves_api_response_structure() {
    let server_url = get_server_url();
    let client = Client::new();

    let res = client
        .get(format!("{server_url}/v1/api/system/health"))
        .send()
        .await
        .expect("health request failed");

    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.expect("response must be JSON");

    assert_api_envelope_shape(&body);

    // Health semantics
    assert_eq!(body["success"], true);
    assert!(body["error"].is_null(), "health should not return error");
    assert!(
        !body["data"].is_null(),
        "health should return data (e.g. status info)"
    );
}

#[tokio::test]
async fn not_found_preserves_api_response_structure() {
    let server_url = get_server_url();
    let client = Client::new();

    let res = client
        .get(format!("{server_url}/this/endpoint/does/not/exist"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(res.status(), 404);

    let body: Value = res.json().await.expect("404 response must be JSON");

    assert_api_envelope_shape(&body);

    // Error semantics
    assert_eq!(body["success"], false);
    assert!(body["data"].is_null(), "data should be null on error");
    assert!(body["error"].is_object(), "error must be present on 404");

    // Optional: assert your NOT_FOUND code if you want it strict
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}
