//! Supabase JWT verification service.
//!
//! Validates Supabase-issued JWTs using JWKS-based key resolution.
//! Performs full OIDC-style validation: signature, issuer, audience, exp, nbf, sub.

use crate::ServiceError;
use jsonwebtoken::{jwk::JwkSet, Algorithm, DecodingKey, TokenData, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Cached JWKS with expiry time.
struct JwksCache {
    jwks: JwkSet,
    fetched_at: std::time::Instant,
}

/// Supabase JWT claims we care about.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_metadata: Option<serde_json::Value>,
}

/// Verified Supabase token result.
pub struct VerifiedSupabaseToken {
    pub user_id: uuid::Uuid,
    pub email: Option<String>,
    pub user_metadata: Option<serde_json::Value>,
}

/// Verifies Supabase JWTs using JWKS.
pub struct SupabaseJwtVerifier {
    jwks_url: String,
    issuer: String,
    audience: String,
    client: Client,
    cache: Arc<tokio::sync::RwLock<Option<JwksCache>>>,
    cache_ttl: Duration,
}

impl SupabaseJwtVerifier {
    pub fn new(jwks_url: String, issuer: String, audience: String) -> Self {
        Self {
            jwks_url,
            issuer,
            audience,
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to build HTTP client"),
            cache: Arc::new(tokio::sync::RwLock::new(None)),
            cache_ttl: Duration::from_secs(300),
        }
    }

    /// Fetch JWKS from the configured URL.
    async fn fetch_jwks(&self) -> Result<JwkSet, ServiceError> {
        let response =
            self.client.get(&self.jwks_url).send().await.map_err(|e| {
                ServiceError::AuthenticationError(format!("JWKS fetch failed: {}", e))
            })?;

        let jwks: JwkSet = response
            .json()
            .await
            .map_err(|e| ServiceError::AuthenticationError(format!("JWKS parse failed: {}", e)))?;

        Ok(jwks)
    }

    /// Get JWKS from cache or fetch fresh.
    async fn get_jwks(&self) -> Result<JwkSet, ServiceError> {
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < self.cache_ttl {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        let jwks = self.fetch_jwks().await?;
        let mut cache = self.cache.write().await;
        *cache = Some(JwksCache {
            jwks: jwks.clone(),
            fetched_at: std::time::Instant::now(),
        });
        Ok(jwks)
    }

    /// Find the JWK by kid and convert to DecodingKey.
    fn find_decoding_key(&self, jwks: &JwkSet, kid: &str) -> Result<DecodingKey, ServiceError> {
        let jwk = jwks
            .keys
            .iter()
            .find(|k| k.common.key_id.as_deref() == Some(kid))
            .ok_or_else(|| {
                ServiceError::AuthenticationError(format!("Unknown key ID (kid): {}", kid))
            })?;

        DecodingKey::from_jwk(jwk)
            .map_err(|e| ServiceError::AuthenticationError(format!("Invalid JWK: {}", e)))
    }

    /// Validate and decode a Supabase JWT.
    pub async fn verify_token(&self, token: &str) -> Result<VerifiedSupabaseToken, ServiceError> {
        let header = jsonwebtoken::decode_header(token).map_err(|e| {
            ServiceError::AuthenticationError(format!("Invalid token header: {}", e))
        })?;

        let kid = header.kid.ok_or_else(|| {
            ServiceError::AuthenticationError("Missing key ID (kid) in token header".to_string())
        })?;

        let jwks = self.get_jwks().await?;
        let decoding_key = self.find_decoding_key(&jwks, &kid)?;

        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.set_required_spec_claims(&["sub"]);

        let token_data: TokenData<SupabaseClaims> =
            decode_with_key(token, &decoding_key, &validation)?;

        let user_id = uuid::Uuid::parse_str(&token_data.claims.sub).map_err(|_| {
            ServiceError::AuthenticationError("Invalid subject format (expected UUID)".to_string())
        })?;

        Ok(VerifiedSupabaseToken {
            user_id,
            email: token_data.claims.email,
            user_metadata: token_data.claims.user_metadata,
        })
    }
}

/// Helper function to decode JWT with a DecodingKey.
fn decode_with_key<T: serde::de::DeserializeOwned>(
    token: &str,
    key: &DecodingKey,
    validation: &Validation,
) -> Result<TokenData<T>, ServiceError> {
    use jsonwebtoken::{decode, errors::ErrorKind};

    match decode(token, key, validation) {
        Ok(data) => Ok(data),
        Err(e) => Err(match e.kind() {
            ErrorKind::ExpiredSignature => {
                ServiceError::ExpiredTokenError("Supabase token expired".to_string())
            }
            ErrorKind::InvalidIssuer => {
                ServiceError::AuthenticationError("Invalid token issuer".to_string())
            }
            ErrorKind::InvalidAudience => {
                ServiceError::AuthenticationError("Invalid token audience".to_string())
            }
            ErrorKind::InvalidSubject => {
                ServiceError::AuthenticationError("Missing or invalid subject".to_string())
            }
            ErrorKind::InvalidSignature => {
                ServiceError::AuthenticationError("Invalid token signature".to_string())
            }
            _ => ServiceError::AuthenticationError(format!("Token validation failed: {}", e)),
        }),
    }
}
