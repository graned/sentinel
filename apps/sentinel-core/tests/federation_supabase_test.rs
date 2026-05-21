//! Integration tests for Supabase federation token exchange.
//!
//! **NOTE:** These tests require the server to be running with:
//! - `SUPABASE_FEDERATION_ENABLED=true`
//! - `SUPABASE_JWKS_URL` pointing to a test JWKS endpoint
//!
//! To run: `cargo test --test federation_supabase_test -- --include-ignored`
//!
//! The tests use an in-memory JWKS server to avoid requiring a real Supabase account.

mod common;

use common::helpers::{assert_error_envelope, post_json, read_json};
use common::jwks_server::{
    build_jwks_server, generate_expired_test_jwt, generate_jwt_missing_sub,
    generate_jwt_unknown_kid, generate_jwt_wrong_audience, generate_jwt_wrong_issuer,
    generate_jwt_wrong_key, generate_test_keypair, generate_test_supabase_jwt, JwksTestState,
};
use reqwest::Client;
use serde_json::json;
use tokio::net::TcpListener;

const SUPABASE_ISSUER: &str = "http://localhost:9999/auth/v1";
const SUPABASE_AUDIENCE: &str = "authenticated";

fn get_supabase_exchange_url() -> String {
    format!(
        "{}/v1/api/federation/supabase/exchange",
        common::setup::get_server_url()
    )
}

/// Test 1: Valid token for existing external identity returns a Sentinel session.
#[tokio::test]
#[ignore]
async fn exchange_existing_identity_returns_session() {
    let (private_key, kid, n, e) = generate_test_keypair();
    let state = JwksTestState {
        private_key: private_key.clone(),
        kid: kid.clone(),
        n,
        e,
    };
    let jwks_app = build_jwks_server(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let jwks_url = format!("http://127.0.0.1:{}/.well-known/jwks.json", port);

    let server = tokio::spawn(async move {
        axum::serve(listener, jwks_app).await.unwrap();
    });

    let test_user_id = uuid::Uuid::new_v4();
    let test_sub = test_user_id.to_string();

    let token = generate_test_supabase_jwt(
        &private_key,
        &kid,
        &test_sub,
        SUPABASE_ISSUER,
        SUPABASE_AUDIENCE,
        Some("test@example.com"),
        None,
    );

    let client = Client::new();
    let res = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token }),
    )
    .await;

    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 200, "Expected 200, got: {}\nbody: {}", status, body);
    assert!(body.get("success").unwrap().as_bool().unwrap());

    let data = body.get("data").unwrap();
    assert!(data.get("access_token").is_some());
    assert!(data.get("refresh_token").is_some());
    assert_eq!(
        data.get("user_id").unwrap().as_str().unwrap(),
        test_sub,
        "user_id should match the Supabase sub"
    );

    server.abort();
}

/// Test 2: Valid token for new identity creates user + identity + session.
#[tokio::test]
#[ignore]
async fn exchange_new_identity_creates_user_and_session() {
    let (private_key, kid, n, e) = generate_test_keypair();
    let state = JwksTestState {
        private_key: private_key.clone(),
        kid: kid.clone(),
        n,
        e,
    };
    let jwks_app = build_jwks_server(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        axum::serve(listener, jwks_app).await.unwrap();
    });

    let new_sub = uuid::Uuid::new_v4().to_string();
    let token = generate_test_supabase_jwt(
        &private_key,
        &kid,
        &new_sub,
        SUPABASE_ISSUER,
        SUPABASE_AUDIENCE,
        Some("newuser@example.com"),
        Some(json!({ "full_name": "Test User" })),
    );

    let client = Client::new();
    let res = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token }),
    )
    .await;

    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 200, "Expected 200, got: {}\nbody: {}", status, body);
    assert!(body.get("success").unwrap().as_bool().unwrap());

    let data = body.get("data").unwrap();
    assert!(data.get("access_token").is_some());
    assert!(data.get("refresh_token").is_some());

    server.abort();
}

/// Test 3: Calling exchange twice with the same Supabase sub reuses the same Sentinel user.
#[tokio::test]
#[ignore]
async fn exchange_same_sub_reuses_user() {
    let (private_key, kid, n, e) = generate_test_keypair();
    let state = JwksTestState {
        private_key: private_key.clone(),
        kid: kid.clone(),
        n,
        e,
    };
    let jwks_app = build_jwks_server(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        axum::serve(listener, jwks_app).await.unwrap();
    });

    let same_sub = uuid::Uuid::new_v4().to_string();

    let token1 = generate_test_supabase_jwt(
        &private_key,
        &kid,
        &same_sub,
        SUPABASE_ISSUER,
        SUPABASE_AUDIENCE,
        Some("first@example.com"),
        None,
    );

    let client = Client::new();
    let res1 = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token1 }),
    )
    .await;

    let (_, body1, _) = read_json(res1).await;
    let user_id_1 = body1
        .get("data")
        .unwrap()
        .get("user_id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let token2 = generate_test_supabase_jwt(
        &private_key,
        &kid,
        &same_sub,
        SUPABASE_ISSUER,
        SUPABASE_AUDIENCE,
        Some("changed@example.com"),
        None,
    );

    let res2 = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token2 }),
    )
    .await;

    let (_, body2, _) = read_json(res2).await;
    let user_id_2 = body2
        .get("data")
        .unwrap()
        .get("user_id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(
        user_id_1, user_id_2,
        "Same sub should return the same user_id on subsequent exchanges"
    );

    server.abort();
}

/// Test 4: Email change in Supabase token does NOT create a new Sentinel user.
#[tokio::test]
#[ignore]
async fn exchange_email_change_does_not_create_new_user() {
    let (private_key, kid, n, e) = generate_test_keypair();
    let state = JwksTestState {
        private_key: private_key.clone(),
        kid: kid.clone(),
        n,
        e,
    };
    let jwks_app = build_jwks_server(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        axum::serve(listener, jwks_app).await.unwrap();
    });

    let identity_sub = uuid::Uuid::new_v4().to_string();

    let token1 = generate_test_supabase_jwt(
        &private_key,
        &kid,
        &identity_sub,
        SUPABASE_ISSUER,
        SUPABASE_AUDIENCE,
        Some("email1@example.com"),
        None,
    );

    let client = Client::new();
    let res1 = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token1 }),
    )
    .await;

    let (_, body1, _) = read_json(res1).await;
    let user_id_1 = body1
        .get("data")
        .unwrap()
        .get("user_id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let token2 = generate_test_supabase_jwt(
        &private_key,
        &kid,
        &identity_sub,
        SUPABASE_ISSUER,
        SUPABASE_AUDIENCE,
        Some("email2@example.com"),
        None,
    );

    let res2 = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token2 }),
    )
    .await;

    let (_, body2, _) = read_json(res2).await;
    let user_id_2 = body2
        .get("data")
        .unwrap()
        .get("user_id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(
        user_id_1, user_id_2,
        "Email change should not create a new user - identity is matched by provider+issuer+subject"
    );

    server.abort();
}

/// Test 5: Invalid signature returns 401.
#[tokio::test]
#[ignore]
async fn exchange_invalid_signature_returns_401() {
    let (private_key, kid, n, e) = generate_test_keypair();
    let (wrong_key, _, _, _) = generate_test_keypair();
    let state = JwksTestState {
        private_key: private_key.clone(),
        kid: kid.clone(),
        n,
        e,
    };
    let jwks_app = build_jwks_server(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        axum::serve(listener, jwks_app).await.unwrap();
    });

    let sub = uuid::Uuid::new_v4().to_string();
    let token = generate_jwt_wrong_key(&wrong_key, &kid, &sub, SUPABASE_ISSUER, SUPABASE_AUDIENCE);

    let client = Client::new();
    let res = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token }),
    )
    .await;

    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 401, "Expected 401 for invalid signature");
    assert_error_envelope(&body, "AUTH_ERROR");

    server.abort();
}

/// Test 6: Expired token returns 401.
#[tokio::test]
#[ignore]
async fn exchange_expired_token_returns_401() {
    let (private_key, kid, n, e) = generate_test_keypair();
    let state = JwksTestState {
        private_key: private_key.clone(),
        kid: kid.clone(),
        n,
        e,
    };
    let jwks_app = build_jwks_server(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        axum::serve(listener, jwks_app).await.unwrap();
    });

    let sub = uuid::Uuid::new_v4().to_string();
    let token =
        generate_expired_test_jwt(&private_key, &kid, &sub, SUPABASE_ISSUER, SUPABASE_AUDIENCE);

    let client = Client::new();
    let res = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token }),
    )
    .await;

    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 401, "Expected 401 for expired token");
    assert_error_envelope(&body, "EXPIRED_TOKEN");

    server.abort();
}

/// Test 7: Wrong issuer returns 401.
#[tokio::test]
#[ignore]
async fn exchange_wrong_issuer_returns_401() {
    let (private_key, kid, n, e) = generate_test_keypair();
    let state = JwksTestState {
        private_key: private_key.clone(),
        kid: kid.clone(),
        n,
        e,
    };
    let jwks_app = build_jwks_server(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        axum::serve(listener, jwks_app).await.unwrap();
    });

    let sub = uuid::Uuid::new_v4().to_string();
    let token = generate_jwt_wrong_issuer(
        &private_key,
        &kid,
        &sub,
        SUPABASE_ISSUER,
        "http://evil.com/auth",
        SUPABASE_AUDIENCE,
    );

    let client = Client::new();
    let res = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token }),
    )
    .await;

    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 401, "Expected 401 for wrong issuer");
    assert_error_envelope(&body, "AUTH_ERROR");

    server.abort();
}

/// Test 8: Wrong audience returns 401.
#[tokio::test]
#[ignore]
async fn exchange_wrong_audience_returns_401() {
    let (private_key, kid, n, e) = generate_test_keypair();
    let state = JwksTestState {
        private_key: private_key.clone(),
        kid: kid.clone(),
        n,
        e,
    };
    let jwks_app = build_jwks_server(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        axum::serve(listener, jwks_app).await.unwrap();
    });

    let sub = uuid::Uuid::new_v4().to_string();
    let token = generate_jwt_wrong_audience(
        &private_key,
        &kid,
        &sub,
        SUPABASE_ISSUER,
        SUPABASE_AUDIENCE,
        "unauthorized",
    );

    let client = Client::new();
    let res = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token }),
    )
    .await;

    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 401, "Expected 401 for wrong audience");
    assert_error_envelope(&body, "AUTH_ERROR");

    server.abort();
}

/// Test 9: Unknown kid returns 401.
#[tokio::test]
#[ignore]
async fn exchange_unknown_kid_returns_401() {
    let (private_key, kid, n, e) = generate_test_keypair();
    let state = JwksTestState {
        private_key: private_key.clone(),
        kid: kid.clone(),
        n,
        e,
    };
    let jwks_app = build_jwks_server(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        axum::serve(listener, jwks_app).await.unwrap();
    });

    let sub = uuid::Uuid::new_v4().to_string();
    let token =
        generate_jwt_unknown_kid(&private_key, &kid, &sub, SUPABASE_ISSUER, SUPABASE_AUDIENCE);

    let client = Client::new();
    let res = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token }),
    )
    .await;

    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 401, "Expected 401 for unknown kid");
    assert_error_envelope(&body, "AUTH_ERROR");

    server.abort();
}

/// Test 10: Missing sub returns 401.
#[tokio::test]
#[ignore]
async fn exchange_missing_sub_returns_401() {
    let (private_key, kid, n, e) = generate_test_keypair();
    let state = JwksTestState {
        private_key: private_key.clone(),
        kid: kid.clone(),
        n,
        e,
    };
    let jwks_app = build_jwks_server(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        axum::serve(listener, jwks_app).await.unwrap();
    });

    let token = generate_jwt_missing_sub(&private_key, &kid, SUPABASE_ISSUER, SUPABASE_AUDIENCE);

    let client = Client::new();
    let res = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": token }),
    )
    .await;

    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 401, "Expected 401 for missing sub");
    assert_error_envelope(&body, "AUTH_ERROR");

    server.abort();
}

/// Test 11: Federation disabled returns 404.
#[tokio::test]
#[ignore]
async fn exchange_federation_disabled_returns_404() {
    let client = Client::new();
    let res = post_json(
        &client,
        get_supabase_exchange_url(),
        json!({ "access_token": "fake_token" }),
    )
    .await;

    let (status, body, _) = read_json(res).await;
    assert_eq!(
        status, 404,
        "Expected 404 when federation is disabled (requires SUPABASE_FEDERATION_ENABLED=false)"
    );
    assert_error_envelope(&body, "FEDERATION_NOT_ENABLED");
}
