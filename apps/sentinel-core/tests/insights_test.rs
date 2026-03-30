mod common;

use common::{
    helpers::{admin_login, assert_error_envelope, post_json, read_json, TEST_PASSWORD},
    setup::{
        get_insights_sessions_url, get_insights_stats_url, get_insights_user_growth_url,
        get_login_user_url, get_register_user_url,
    },
};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

// ── Setup helpers ─────────────────────────────────────────────────────────────

async fn register_and_login(client: &Client) -> String {
    let email = format!("insights-user-{}@example.com", Uuid::new_v4());

    let res = post_json(
        client,
        get_register_user_url(),
        json!({
            "first_name": "Insights",
            "last_name":  "Tester",
            "email":      email,
            "password":   TEST_PASSWORD
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "register failed: {raw}");

    let res = post_json(
        client,
        get_login_user_url(),
        json!({ "email": email, "password": TEST_PASSWORD }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");

    body.pointer("/data/access_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing access_token: {body}"))
        .to_string()
}

// ── GET /v1/api/system/stats ───────────────────────────────────────────────────

#[tokio::test]
async fn stats_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .get(get_insights_stats_url())
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

#[tokio::test]
async fn stats_with_expired_token_returns_401() {
    use common::helpers::generate_expired_token;

    let client = Client::new();
    let token = generate_expired_token(Uuid::new_v4(), Uuid::new_v4())
        .expect("failed to generate expired token");

    let res = client
        .get(get_insights_stats_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "EXPIRED_TOKEN");
}

#[tokio::test]
async fn stats_with_non_admin_user_returns_403() {
    let client = Client::new();
    let token = register_and_login(&client).await;

    let res = client
        .get(get_insights_stats_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

// ── GET /v1/api/system/analytics/user-growth ─────────────────────────────────

#[tokio::test]
async fn user_growth_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .get(get_insights_user_growth_url())
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

#[tokio::test]
async fn user_growth_with_expired_token_returns_401() {
    use common::helpers::generate_expired_token;

    let client = Client::new();
    let token = generate_expired_token(Uuid::new_v4(), Uuid::new_v4())
        .expect("failed to generate expired token");

    let res = client
        .get(get_insights_user_growth_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "EXPIRED_TOKEN");
}

#[tokio::test]
async fn user_growth_with_non_admin_user_returns_403() {
    let client = Client::new();
    let token = register_and_login(&client).await;

    let res = client
        .get(get_insights_user_growth_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

// ── GET /v1/api/system/analytics/sessions ────────────────────────────────────

#[tokio::test]
async fn sessions_activity_without_auth_returns_401() {
    let client = Client::new();

    let res = client
        .get(get_insights_sessions_url())
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "MISSING_TOKEN");
}

#[tokio::test]
async fn sessions_activity_with_expired_token_returns_401() {
    use common::helpers::generate_expired_token;

    let client = Client::new();
    let token = generate_expired_token(Uuid::new_v4(), Uuid::new_v4())
        .expect("failed to generate expired token");

    let res = client
        .get(get_insights_sessions_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 401, "expected 401, got {status}\n{raw}");
    assert_error_envelope(&body, "EXPIRED_TOKEN");
}

#[tokio::test]
async fn sessions_activity_with_non_admin_user_returns_403() {
    let client = Client::new();
    let token = register_and_login(&client).await;

    let res = client
        .get(get_insights_sessions_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 403, "expected 403, got {status}\n{raw}");
    assert_error_envelope(&body, "FORBIDDEN");
}

// ── Admin happy-path tests ────────────────────────────────────────────────────

/// GET /v1/api/system/stats as admin → 200 with all KPI fields present.
#[tokio::test]
async fn stats_as_admin_returns_summary_shape() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let res = client
        .get(get_insights_stats_url())
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200, got {status}\n{raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = &body["data"];
    assert!(
        data["total_users"].is_number(),
        "missing total_users: {body}"
    );
    assert!(
        data["new_users_week"].is_number(),
        "missing new_users_week: {body}"
    );
    assert!(
        data["new_users_month"].is_number(),
        "missing new_users_month: {body}"
    );
    assert!(
        data["active_users_week"].is_number(),
        "missing active_users_week: {body}"
    );
    assert!(
        data["active_users_month"].is_number(),
        "missing active_users_month: {body}"
    );
    assert!(
        data["active_sessions"].is_number(),
        "missing active_sessions: {body}"
    );
    assert!(
        data["mfa_adoption_pct"].is_number(),
        "missing mfa_adoption_pct: {body}"
    );
    assert!(
        data["email_verified_pct"].is_number(),
        "missing email_verified_pct: {body}"
    );

    // Sanity: at least 1 user (the seeded admin)
    assert!(
        data["total_users"].as_i64().unwrap_or(0) >= 1,
        "total_users must be >= 1: {body}"
    );
}

/// GET /v1/api/system/analytics/user-growth?days=7 as admin → 200, array of
/// objects each with date, total_users, new_users. The endpoint returns only
/// days with activity, so the count may be less than the requested window.
#[tokio::test]
async fn user_growth_as_admin_returns_array_shape() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let res = client
        .get(format!("{}?days=7", get_insights_user_growth_url()))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200, got {status}\n{raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = body["data"]
        .as_array()
        .unwrap_or_else(|| panic!("data must be array: {body}"));

    // At least one data point must exist (the seeded admin was created)
    assert!(!data.is_empty(), "expected at least one data point: {body}");

    // Every returned point must have the correct fields
    for point in data {
        assert!(point["date"].is_string(), "missing date in point: {point}");
        assert!(
            point["total_users"].is_number(),
            "missing total_users in point: {point}"
        );
        assert!(
            point["new_users"].is_number(),
            "missing new_users in point: {point}"
        );
    }
}

/// GET /v1/api/system/analytics/sessions?days=7 as admin → 200, array of
/// objects each with date, sessions_created, unique_users. The endpoint returns
/// only days with activity, so the count may be less than the requested window.
#[tokio::test]
async fn sessions_activity_as_admin_returns_array_shape() {
    let client = Client::new();
    let token = admin_login(&client).await;

    let res = client
        .get(format!("{}?days=7", get_insights_sessions_url()))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    let (status, body, raw) = read_json(res).await;

    assert_eq!(status, 200, "expected 200, got {status}\n{raw}");
    assert_eq!(body["success"], true, "{body}");

    let data = body["data"]
        .as_array()
        .unwrap_or_else(|| panic!("data must be array: {body}"));

    // At least one data point must exist (sessions are created by the tests themselves)
    assert!(!data.is_empty(), "expected at least one data point: {body}");

    // Every returned point must have the correct fields
    for point in data {
        assert!(point["date"].is_string(), "missing date in point: {point}");
        assert!(
            point["sessions_created"].is_number(),
            "missing sessions_created in point: {point}"
        );
        assert!(
            point["unique_users"].is_number(),
            "missing unique_users in point: {point}"
        );
    }
}
