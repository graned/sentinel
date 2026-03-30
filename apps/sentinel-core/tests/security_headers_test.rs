mod common;
use common::setup::get_server_url;
use reqwest::Client;

#[tokio::test]
async fn security_headers_are_present_on_health_endpoint() {
    let client = Client::new();
    let url = format!("{}/v1/api/system/health", get_server_url());

    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let headers = res.headers();

    assert_eq!(
        headers
            .get("x-content-type-options")
            .map(|v| v.to_str().unwrap()),
        Some("nosniff"),
        "x-content-type-options header missing or wrong"
    );

    assert_eq!(
        headers.get("x-frame-options").map(|v| v.to_str().unwrap()),
        Some("DENY"),
        "x-frame-options header missing or wrong"
    );

    assert_eq!(
        headers.get("referrer-policy").map(|v| v.to_str().unwrap()),
        Some("strict-origin-when-cross-origin"),
        "referrer-policy header missing or wrong"
    );
}
