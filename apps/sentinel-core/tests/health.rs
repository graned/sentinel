mod common;
use common::setup::get_server_url;
use reqwest::Client;

#[tokio::test]
async fn health_check_works() {
    let server_url = get_server_url();
    let api_endpoint = "/v1/api/system/health".to_string();
    let client = Client::new();
    let res = client
        .get(format!("{server_url}{api_endpoint}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}
