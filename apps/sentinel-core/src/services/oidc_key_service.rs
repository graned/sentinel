//! OIDC signing key service — RSA key generation, encryption, and JWKS publication.
//!
//! Sentinel acts as an RS256 JWT signer.  Private keys are stored encrypted at rest
//! in `oidc_signing_keys.private_key_encrypted` using **XChaCha20-Poly1305** with a
//! random 24-byte nonce prepended to the ciphertext (`[nonce || ciphertext]`).
//! The key material is derived from `CONFIG_ENCRYPTION_KEY`.
//!
//! # Key format
//!
//! RSA 2048-bit private keys are serialized as **PKCS#1 DER** before encryption.
//! `ring`'s `RsaKeyPair::from_der()` (used internally by `jsonwebtoken`) requires
//! PKCS#1 DER — **not** PKCS#8.  Use `rsa::pkcs1::EncodeRsaPrivateKey::to_pkcs1_der()`.
//!
//! # Key rotation
//!
//! `generate_and_store_key` retires all previously active keys (sets `status = "retired"`)
//! before inserting the new one.  The JWKS endpoint always returns the single active key.
//!
//! # JWK
//!
//! The public JWK is built from the RSA modulus `n` and exponent `e`, both encoded as
//! URL-safe base64 without padding, and stored as `jsonb` in `public_jwk_json`.

use crate::{DbConnection, OidcSigningKey, OidcSigningKeyRepository, ServiceError};
use base64::Engine;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use rand::rngs::OsRng;
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use std::sync::Arc;
use uuid::Uuid;

/// Byte size of an XChaCha20-Poly1305 nonce (192-bit / 24-byte).
const NONCE_SIZE: usize = 24;

/// Manages OIDC RSA signing key lifecycle (generation, encryption, retrieval, JWKS).
pub struct OidcKeyService {
    key_repository: Arc<OidcSigningKeyRepository>,
    encryption_key: [u8; 32],
}

impl OidcKeyService {
    pub fn new(key_repository: Arc<OidcSigningKeyRepository>, encryption_key: [u8; 32]) -> Self {
        Self {
            key_repository,
            encryption_key,
        }
    }

    /// Generate a new RSA-2048 key pair, encrypt the private key, retire the previous
    /// active key, and persist the new key.  Returns the stored record including the `kid`.
    pub async fn generate_and_store_key(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<OidcSigningKey, ServiceError> {
        // Generate RSA key pair, serialize, and build JWK in a sync block
        // so that OsRng (and the private key) are not held across .await points.
        let (private_key_encrypted, public_jwk, kid) = {
            let private_key = RsaPrivateKey::new(&mut OsRng, 2048)
                .map_err(|e| ServiceError::OidcSigningError(e.to_string()))?;

            // Serialize private key as PKCS1 DER (required by ring/jsonwebtoken from_rsa_der)
            let private_der = private_key
                .to_pkcs1_der()
                .map_err(|e| ServiceError::OidcSigningError(e.to_string()))?
                .as_bytes()
                .to_vec();

            // Build JWK for public key
            let pub_key = private_key.to_public_key();
            let n_bytes = pub_key.n().to_bytes_be();
            let e_bytes = pub_key.e().to_bytes_be();
            let n_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&n_bytes);
            let e_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&e_bytes);

            let kid = Uuid::new_v4().to_string();
            let public_jwk = serde_json::json!({
                "kty": "RSA",
                "use": "sig",
                "alg": "RS256",
                "kid": kid,
                "n": n_b64,
                "e": e_b64,
            });

            let encrypted = self.encrypt_key_bytes(&private_der)?;
            (encrypted, public_jwk, kid)
        };

        // Retire any previous active key
        self.key_repository
            .retire_all_active(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Store new key
        let signing_key = OidcSigningKey {
            oidc_signing_key_id: Uuid::new_v4(),
            kid,
            alg: "RS256".to_string(),
            public_jwk_json: public_jwk,
            private_key_encrypted,
            status: "active".to_string(),
            created_at: chrono::Utc::now(),
        };

        let stored = self
            .key_repository
            .create(conn, &signing_key)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(stored)
    }

    /// Fetch the currently active signing key.  Returns `OidcNoActiveSigningKey` if no
    /// active key exists — the admin must call `POST /admin/oidc/keys/generate` first.
    pub async fn get_active_key(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<OidcSigningKey, ServiceError> {
        self.key_repository
            .find_active(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                ServiceError::OidcNoActiveSigningKey(
                    "No active OIDC signing key found".to_string(),
                )
            })
    }

    /// Decrypt the stored private key DER bytes.
    /// Expects the input to be `[24-byte nonce || XChaCha20-Poly1305 ciphertext]`.
    pub fn decrypt_private_key(&self, encrypted: &[u8]) -> Result<Vec<u8>, ServiceError> {
        if encrypted.len() < NONCE_SIZE {
            return Err(ServiceError::OidcSigningError(
                "Invalid encrypted key data".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
        let nonce = XNonce::from_slice(nonce_bytes);

        let cipher = XChaCha20Poly1305::new_from_slice(&self.encryption_key)
            .map_err(|_| ServiceError::InternalError("Invalid encryption key".to_string()))?;

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ServiceError::OidcSigningError(e.to_string()))
    }

    /// Build the JWKS document (`{ "keys": [<active JWK>] }`) for the `GET /oauth/jwks.json`
    /// endpoint.  Returns an empty `keys` array when no active key exists.
    pub async fn get_jwks(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<serde_json::Value, ServiceError> {
        let active_key = self.key_repository
            .find_active(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let keys = match active_key {
            Some(key) => vec![key.public_jwk_json],
            None => vec![],
        };

        Ok(serde_json::json!({ "keys": keys }))
    }

    /// Encrypt `data` with XChaCha20-Poly1305 using a freshly generated random nonce.
    /// Returns `[nonce (24 bytes) || ciphertext]`.
    fn encrypt_key_bytes(&self, data: &[u8]) -> Result<Vec<u8>, ServiceError> {
        let cipher = XChaCha20Poly1305::new_from_slice(&self.encryption_key)
            .map_err(|_| ServiceError::InternalError("Invalid encryption key".to_string()))?;

        let nonce: XNonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| ServiceError::OidcSigningError(e.to_string()))?;

        let nonce_bytes: [u8; NONCE_SIZE] = nonce.into();
        let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);

        Ok(output)
    }
}
