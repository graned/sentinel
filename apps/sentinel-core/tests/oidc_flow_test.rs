mod common;

use base64::Engine;
use common::{
    helpers::{admin_login, post_json, read_json},
    setup::{
        get_jwks_url, get_login_user_url, get_oauth_authorize_url, get_oauth_token_url,
        get_oidc_create_client_url, get_oidc_discovery_url, get_oidc_generate_key_url,
        get_register_user_url,
    },
};
use dotenvy::dotenv;
use rand::Rng;
use reqwest::redirect;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

async fn mark_email_verified(email: &str) {
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
            "UPDATE user_identities SET email_verified = true WHERE email = $1",
            &[&email],
        )
        .await
        .expect("DB update failed");
}

// ── PKCE helpers ──────────────────────────────────────────────────────────────

fn generate_code_verifier() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn compute_code_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash.as_slice())
}

/// Parse query string from a URL into a key-value map.
fn parse_query_params(url: &str) -> HashMap<String, String> {
    url.split('?')
        .nth(1)
        .unwrap_or("")
        .split('&')
        .filter_map(|part| {
            let mut kv = part.splitn(2, '=');
            let k = kv.next()?.to_string();
            let v = kv.next().unwrap_or("").to_string();
            Some((k, v))
        })
        .collect()
}

// ── Test ──────────────────────────────────────────────────────────────────────

/// Full end-to-end OIDC Authorization Code + PKCE flow.
#[tokio::test]
async fn oidc_full_flow() {
    // Client that does NOT follow redirects — needed for /oauth/authorize
    let no_redirect_client = reqwest::ClientBuilder::new()
        .redirect(redirect::Policy::none())
        .build()
        .expect("failed to build no-redirect client");

    let client = reqwest::Client::new();
    let admin_token = admin_login(&client).await;

    // ── Step 1: Generate a signing key ────────────────────────────────────────
    let res = client
        .post(get_oidc_generate_key_url())
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("generate key request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "generate signing key failed: {raw}");
    assert_eq!(body["success"], true, "generate key response: {body}");
    let kid = body["data"]["kid"]
        .as_str()
        .unwrap_or_else(|| panic!("missing kid: {body}"))
        .to_string();
    assert!(!kid.is_empty(), "kid should not be empty");
    assert_eq!(body["data"]["alg"], "RS256", "expected RS256 alg: {body}");
    assert_eq!(
        body["data"]["status"], "active",
        "expected active status: {body}"
    );

    // ── Step 2: Create an OIDC client ─────────────────────────────────────────
    let client_id = format!("test-app-{}", Uuid::new_v4());
    let redirect_uri = "http://localhost:3000/callback";

    let res = client
        .post(get_oidc_create_client_url())
        .bearer_auth(&admin_token)
        .json(&json!({
            "client_id": client_id,
            "name": "Test Application",
            "redirect_uris": [redirect_uri],
            "allowed_scopes": ["openid", "email"],
            "is_confidential": false,
            "pkce_required": true
        }))
        .send()
        .await
        .expect("create OIDC client request failed");
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "create client failed: {raw}");
    assert_eq!(body["success"], true, "create client response: {body}");
    assert_eq!(
        body["data"]["client_id"], client_id,
        "client_id mismatch: {body}"
    );
    assert!(
        !body["data"]["oidc_client_id"].is_null(),
        "missing oidc_client_id: {body}"
    );

    // ── Step 3: Register user + login ─────────────────────────────────────────
    let email = format!("oidc-test-{}@example.com", Uuid::new_v4());
    let password = "SecurePassword123!";

    let res = post_json(
        &client,
        get_register_user_url(),
        json!({
            "first_name": "OIDC",
            "last_name": "User",
            "email": email,
            "password": password
        }),
    )
    .await;
    let (status, _, raw) = read_json(res).await;
    assert_eq!(status, 200, "registration failed: {raw}");

    // Pre-verify email so the PASETO token has ev=true; /oauth/authorize goes
    // through authorize_middleware which blocks unverified users.
    mark_email_verified(&email).await;

    let res = post_json(
        &client,
        get_login_user_url(),
        json!({ "email": email, "password": password }),
    )
    .await;
    let (status, body, raw) = read_json(res).await;
    assert_eq!(status, 200, "login failed: {raw}");
    let paseto_token = body["data"]["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("no access_token: {raw}"))
        .to_string();

    // ── Step 4: Generate PKCE pair ────────────────────────────────────────────
    let code_verifier = generate_code_verifier();
    let code_challenge = compute_code_challenge(&code_verifier);
    let state = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();

    // ── Step 5: GET /oauth/authorize ─────────────────────────────────────────
    // reqwest .query() handles URL encoding automatically
    let res = no_redirect_client
        .get(get_oauth_authorize_url())
        .query(&[
            ("response_type", "code"),
            ("client_id", client_id.as_str()),
            ("redirect_uri", redirect_uri),
            ("scope", "openid email"),
            ("state", state.as_str()),
            ("nonce", nonce.as_str()),
            ("code_challenge", code_challenge.as_str()),
            ("code_challenge_method", "S256"),
        ])
        .bearer_auth(&paseto_token)
        .send()
        .await
        .expect("authorize request failed");

    // Should be a redirect (3xx)
    let status = res.status().as_u16();
    assert!(
        (300..400).contains(&status),
        "expected 3xx redirect from /oauth/authorize, got {status}"
    );

    let location = res
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_else(|| panic!("missing Location header in authorize response"))
        .to_string();

    // Extract code from Location header query params
    let params = parse_query_params(&location);
    let code = params
        .get("code")
        .cloned()
        .unwrap_or_else(|| panic!("missing 'code' in Location: {location}"));
    let returned_state = params
        .get("state")
        .cloned()
        .unwrap_or_else(|| panic!("missing 'state' in Location: {location}"));

    assert_eq!(returned_state, state, "state mismatch in Location header");
    assert!(!code.is_empty(), "authorization code should not be empty");

    // ── Step 6: POST /oauth/token ─────────────────────────────────────────────
    let res = client
        .post(get_oauth_token_url())
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", redirect_uri),
            ("client_id", client_id.as_str()),
            ("code_verifier", code_verifier.as_str()),
        ])
        .send()
        .await
        .expect("token exchange request failed");

    let (status, token_body, raw) = read_json(res).await;
    assert_eq!(status, 200, "token exchange failed: {raw}");

    let access_token = token_body["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("missing access_token in token response: {raw}"))
        .to_string();
    let id_token = token_body["id_token"]
        .as_str()
        .unwrap_or_else(|| panic!("missing id_token in token response: {raw}"))
        .to_string();
    assert_eq!(
        token_body["token_type"], "Bearer",
        "token_type mismatch: {raw}"
    );
    assert!(
        token_body["expires_in"].as_u64().unwrap_or(0) > 0,
        "expires_in should be positive: {raw}"
    );
    assert!(!access_token.is_empty(), "access_token should not be empty");
    assert!(!id_token.is_empty(), "id_token should not be empty");

    // ── Step 7: Decode id_token JWT (claims only, no signature verify) ────────
    let parts: Vec<&str> = id_token.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "id_token should have 3 JWT parts (header.payload.signature)"
    );

    let claims_b64 = parts[1];
    let claims_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(claims_b64)
        .unwrap_or_else(|e| panic!("failed to decode id_token claims: {e}"));
    let claims: serde_json::Value = serde_json::from_slice(&claims_bytes)
        .unwrap_or_else(|e| panic!("failed to parse id_token claims JSON: {e}"));

    assert!(
        claims["sub"].as_str().is_some(),
        "id_token missing 'sub' claim: {claims}"
    );
    assert_eq!(
        claims["aud"].as_str().unwrap_or(""),
        client_id,
        "id_token 'aud' should match client_id: {claims}"
    );
    assert!(
        claims["iss"].as_str().is_some(),
        "id_token missing 'iss' claim: {claims}"
    );
    assert_eq!(
        claims["email"].as_str().unwrap_or(""),
        email,
        "id_token 'email' should match user email: {claims}"
    );

    // ── Step 8: GET /oauth/jwks.json ─────────────────────────────────────────
    let res = client
        .get(get_jwks_url())
        .send()
        .await
        .expect("jwks request failed");
    let (status, jwks_body, raw) = read_json(res).await;
    assert_eq!(status, 200, "JWKS request failed: {raw}");

    let keys = jwks_body["keys"]
        .as_array()
        .unwrap_or_else(|| panic!("JWKS missing 'keys' array: {raw}"));
    assert!(
        !keys.is_empty(),
        "JWKS 'keys' array should not be empty: {raw}"
    );

    let first_key = &keys[0];
    assert!(
        first_key["kid"].as_str().is_some(),
        "JWK missing 'kid': {first_key}"
    );
    assert_eq!(
        first_key["kty"], "RSA",
        "JWK 'kty' should be RSA: {first_key}"
    );
    assert_eq!(
        first_key["alg"], "RS256",
        "JWK 'alg' should be RS256: {first_key}"
    );
    assert!(
        first_key["n"].as_str().is_some(),
        "JWK missing modulus 'n': {first_key}"
    );
    assert!(
        first_key["e"].as_str().is_some(),
        "JWK missing exponent 'e': {first_key}"
    );

    // ── Step 9: GET /.well-known/openid-configuration ─────────────────────────
    let res = client
        .get(get_oidc_discovery_url())
        .send()
        .await
        .expect("discovery request failed");
    let (status, discovery_body, raw) = read_json(res).await;
    assert_eq!(status, 200, "discovery request failed: {raw}");

    assert!(
        discovery_body["issuer"].as_str().is_some(),
        "discovery missing 'issuer': {raw}"
    );
    assert!(
        discovery_body["authorization_endpoint"].as_str().is_some(),
        "discovery missing 'authorization_endpoint': {raw}"
    );
    assert!(
        discovery_body["token_endpoint"].as_str().is_some(),
        "discovery missing 'token_endpoint': {raw}"
    );
    assert!(
        discovery_body["jwks_uri"].as_str().is_some(),
        "discovery missing 'jwks_uri': {raw}"
    );
    assert!(
        discovery_body["response_types_supported"]
            .as_array()
            .is_some(),
        "discovery missing 'response_types_supported': {raw}"
    );
    assert!(
        discovery_body["id_token_signing_alg_values_supported"]
            .as_array()
            .is_some(),
        "discovery missing 'id_token_signing_alg_values_supported': {raw}"
    );

    let supported_algos = discovery_body["id_token_signing_alg_values_supported"]
        .as_array()
        .unwrap();
    assert!(
        supported_algos.iter().any(|v| v.as_str() == Some("RS256")),
        "RS256 should be in id_token_signing_alg_values_supported: {raw}"
    );
}
