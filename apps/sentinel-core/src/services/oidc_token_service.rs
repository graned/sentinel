//! OIDC token service — RS256 JWT minting for the OIDC protocol.
//!
//! Sentinel issues two JWT types for OIDC flows (distinct from PASETO session tokens):
//!
//! | Token | Claims |
//! |-------|--------|
//! | ID token | `iss`, `sub`, `aud`, `exp`, `iat`, `email`, `nonce?`, `amr?` |
//! | Access token | `iss`, `sub`, `aud`, `scope`, `exp`, `iat` |
//!
//! Both are signed with **RS256** using the active RSA-2048 key stored in
//! `oidc_signing_keys`.  The `kid` header field in each JWT maps to the key's UUID,
//! allowing relying parties to select the correct public key from the JWKS endpoint.
//!
//! # `amr` claim
//!
//! The ID token includes `amr` (Authentication Methods References) to indicate how
//! the user authenticated: `["pwd"]` for password-only, `["pwd", "totp"]` when MFA
//! was used.  This is determined by `MfaTotpService::is_mfa_enabled` at token-issue
//! time.

use crate::{OidcSigningKey, ServiceError, User, UserIdentity};
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Signs OIDC ID tokens and access tokens using the active RS256 key.
pub struct OidcTokenService {
    issuer: String,
}

impl OidcTokenService {
    pub fn new(issuer: String) -> Self {
        Self { issuer }
    }

    pub fn issuer_url(&self) -> &str {
        &self.issuer
    }

    /// Mint a signed OIDC ID token for the given user.
    ///
    /// The `decrypted_private_der` must be PKCS#1 DER bytes from
    /// [`OidcKeyService::decrypt_private_key`].  The token is signed with RS256 and
    /// the `kid` header is set from `signing_key.kid`.
    pub fn generate_id_token(
        &self,
        signing_key: &OidcSigningKey,
        decrypted_private_der: &[u8],
        user: &User,
        identity: &UserIdentity,
        client_id: &str,
        nonce: Option<&str>,
        exp_secs: u64,
        amr: Option<Vec<String>>,
    ) -> Result<String, ServiceError> {
        let now = Utc::now().timestamp() as u64;
        let claims = IdTokenClaims {
            iss: self.issuer.clone(),
            sub: user.user_id.to_string(),
            aud: client_id.to_string(),
            exp: now + exp_secs,
            iat: now,
            nonce: nonce.map(|s| s.to_string()),
            email: identity.email.clone(),
            amr,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(signing_key.kid.clone());

        let encoding_key = EncodingKey::from_rsa_der(decrypted_private_der);

        encode(&header, &claims, &encoding_key)
            .map_err(|e| ServiceError::OidcSigningError(e.to_string()))
    }

    /// Mint a signed OIDC access token for the given user + client + scope.
    pub fn generate_access_token(
        &self,
        signing_key: &OidcSigningKey,
        decrypted_private_der: &[u8],
        user_id: Uuid,
        client_id: &str,
        scope: &str,
        exp_secs: u64,
    ) -> Result<String, ServiceError> {
        let now = Utc::now().timestamp() as u64;
        let claims = AccessTokenClaims {
            iss: self.issuer.clone(),
            sub: user_id.to_string(),
            aud: client_id.to_string(),
            scope: scope.to_string(),
            exp: now + exp_secs,
            iat: now,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(signing_key.kid.clone());

        let encoding_key = EncodingKey::from_rsa_der(decrypted_private_der);

        encode(&header, &claims, &encoding_key)
            .map_err(|e| ServiceError::OidcSigningError(e.to_string()))
    }
}

/// JWT claims for an OIDC ID token (OpenID Connect Core § 3.1.3.3).
#[derive(Debug, Serialize, Deserialize)]
struct IdTokenClaims {
    iss: String,
    sub: String,
    aud: String,
    exp: u64,
    iat: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    nonce: Option<String>,
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    amr: Option<Vec<String>>,
}

/// JWT claims for an OIDC access token.
#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenClaims {
    iss: String,
    sub: String,
    aud: String,
    scope: String,
    exp: u64,
    iat: u64,
}
