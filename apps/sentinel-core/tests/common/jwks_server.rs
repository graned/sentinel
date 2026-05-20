//! Test helper: in-memory JWKS server for Supabase federation tests.

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rsa::{pkcs8::EncodePrivateKey, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test JWKS state.
#[derive(Clone)]
pub struct JwksTestState {
    pub private_key: RsaPrivateKey,
    pub kid: String,
    pub n: String,
    pub e: String,
}

/// JWK representation.
#[derive(Serialize, Deserialize)]
pub struct TestJwk {
    pub kty: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub alg: String,
    pub kid: String,
    pub n: String,
    pub e: String,
}

/// JWKS response.
#[derive(Serialize, Deserialize)]
pub struct TestJwkSet {
    pub keys: Vec<TestJwk>,
}

/// Generate a test RSA key pair and build JWK components.
pub fn generate_test_keypair() -> (RsaPrivateKey, String, String, String) {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use rand::rngs::OsRng;
    use rsa::traits::PublicKeyParts;

    let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
    let pub_key = private_key.to_public_key();

    let kid = uuid::Uuid::new_v4().to_string();
    let n = URL_SAFE_NO_PAD.encode(pub_key.n().to_bytes_be());
    let e = URL_SAFE_NO_PAD.encode(pub_key.e().to_bytes_be());

    (private_key, kid, n, e)
}

/// Build a test JWKS router.
pub fn build_jwks_server(state: JwksTestState) -> Router {
    Router::new()
        .route("/.well-known/jwks.json", get(get_jwks))
        .with_state(Arc::new(RwLock::new(state)))
}

async fn get_jwks(State(state): State<Arc<RwLock<JwksTestState>>>) -> Json<TestJwkSet> {
    let state = state.read().await;
    Json(TestJwkSet {
        keys: vec![TestJwk {
            kty: "RSA".to_string(),
            use_: "sig".to_string(),
            alg: "RS256".to_string(),
            kid: state.kid.clone(),
            n: state.n.clone(),
            e: state.e.clone(),
        }],
    })
}

/// Generate a test Supabase JWT.
pub fn generate_test_supabase_jwt(
    private_key: &RsaPrivateKey,
    kid: &str,
    sub: &str,
    issuer: &str,
    audience: &str,
    email: Option<&str>,
    user_metadata: Option<serde_json::Value>,
) -> String {
    use chrono::{Duration, Utc};
    use serde_json::json;

    let now = Utc::now().timestamp() as u64;
    let exp = (Utc::now() + Duration::minutes(5)).timestamp() as u64;

    let claims = json!({
        "sub": sub,
        "iss": issuer,
        "aud": audience,
        "exp": exp,
        "iat": now,
        "email": email,
        "user_metadata": user_metadata.unwrap_or_else(|| json!({})),
    });

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());

    use rsa::pkcs1::EncodeRsaPrivateKey;
    let der = private_key.to_pkcs1_der().unwrap();
    let encoding_key = EncodingKey::from_rsa_der(der.as_bytes());

    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

/// Generate an expired test JWT.
pub fn generate_expired_test_jwt(
    private_key: &RsaPrivateKey,
    kid: &str,
    sub: &str,
    issuer: &str,
    audience: &str,
) -> String {
    use chrono::{Duration, Utc};
    use serde_json::json;

    let now = Utc::now().timestamp() as u64;
    let exp = (Utc::now() - Duration::minutes(5)).timestamp() as u64;

    let claims = json!({
        "sub": sub,
        "iss": issuer,
        "aud": audience,
        "exp": exp,
        "iat": now - 600,
    });

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());

    use rsa::pkcs1::EncodeRsaPrivateKey;
    let der = private_key.to_pkcs1_der().unwrap();
    let encoding_key = EncodingKey::from_rsa_der(der.as_bytes());

    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

/// Generate a test JWT with wrong issuer.
pub fn generate_jwt_wrong_issuer(
    private_key: &RsaPrivateKey,
    kid: &str,
    sub: &str,
    _correct_issuer: &str,
    wrong_issuer: &str,
    audience: &str,
) -> String {
    use chrono::{Duration, Utc};
    use serde_json::json;

    let now = Utc::now().timestamp() as u64;
    let exp = (Utc::now() + Duration::minutes(5)).timestamp() as u64;

    let claims = json!({
        "sub": sub,
        "iss": wrong_issuer,
        "aud": audience,
        "exp": exp,
        "iat": now,
    });

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());

    use rsa::pkcs1::EncodeRsaPrivateKey;
    let der = private_key.to_pkcs1_der().unwrap();
    let encoding_key = EncodingKey::from_rsa_der(der.as_bytes());

    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

/// Generate a test JWT with wrong audience.
pub fn generate_jwt_wrong_audience(
    private_key: &RsaPrivateKey,
    kid: &str,
    sub: &str,
    issuer: &str,
    _correct_audience: &str,
    wrong_audience: &str,
) -> String {
    use chrono::{Duration, Utc};
    use serde_json::json;

    let now = Utc::now().timestamp() as u64;
    let exp = (Utc::now() + Duration::minutes(5)).timestamp() as u64;

    let claims = json!({
        "sub": sub,
        "iss": issuer,
        "aud": wrong_audience,
        "exp": exp,
        "iat": now,
    });

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());

    use rsa::pkcs1::EncodeRsaPrivateKey;
    let der = private_key.to_pkcs1_der().unwrap();
    let encoding_key = EncodingKey::from_rsa_der(der.as_bytes());

    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

/// Generate a test JWT with unknown kid.
pub fn generate_jwt_unknown_kid(
    private_key: &RsaPrivateKey,
    _correct_kid: &str,
    sub: &str,
    issuer: &str,
    audience: &str,
) -> String {
    use chrono::{Duration, Utc};
    use serde_json::json;

    let now = Utc::now().timestamp() as u64;
    let exp = (Utc::now() + Duration::minutes(5)).timestamp() as u64;

    let claims = json!({
        "sub": sub,
        "iss": issuer,
        "aud": audience,
        "exp": exp,
        "iat": now,
    });

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("unknown-kid-12345".to_string());

    use rsa::pkcs1::EncodeRsaPrivateKey;
    let der = private_key.to_pkcs1_der().unwrap();
    let encoding_key = EncodingKey::from_rsa_der(der.as_bytes());

    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

/// Generate a test JWT without sub claim.
pub fn generate_jwt_missing_sub(
    private_key: &RsaPrivateKey,
    kid: &str,
    issuer: &str,
    audience: &str,
) -> String {
    use chrono::{Duration, Utc};
    use serde_json::json;

    let now = Utc::now().timestamp() as u64;
    let exp = (Utc::now() + Duration::minutes(5)).timestamp() as u64;

    let claims = json!({
        "iss": issuer,
        "aud": audience,
        "exp": exp,
        "iat": now,
    });

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());

    use rsa::pkcs1::EncodeRsaPrivateKey;
    let der = private_key.to_pkcs1_der().unwrap();
    let encoding_key = EncodingKey::from_rsa_der(der.as_bytes());

    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

/// Generate a test JWT signed with wrong key.
pub fn generate_jwt_wrong_key(
    wrong_private_key: &RsaPrivateKey,
    kid: &str,
    sub: &str,
    issuer: &str,
    audience: &str,
) -> String {
    use chrono::{Duration, Utc};
    use serde_json::json;

    let now = Utc::now().timestamp() as u64;
    let exp = (Utc::now() + Duration::minutes(5)).timestamp() as u64;

    let claims = json!({
        "sub": sub,
        "iss": issuer,
        "aud": audience,
        "exp": exp,
        "iat": now,
    });

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());

    use rsa::pkcs1::EncodeRsaPrivateKey;
    let der = wrong_private_key.to_pkcs1_der().unwrap();
    let encoding_key = EncodingKey::from_rsa_der(der.as_bytes());

    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}
