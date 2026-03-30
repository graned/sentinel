//! SMTP provider configuration management — encryption, decryption, and redaction.
//!
//! Provider configurations (host, port, credentials) contain secrets that must
//! never be stored in plain text. This service wraps all CRUD operations and
//! enforces the following invariant:
//!
//! - The `config_encrypted` column in `provider_configurations` always holds the
//!   XChaCha20-Poly1305 ciphertext of the full JSON config.
//! - The `config_redacted` column holds a version with **all leaf values** replaced
//!   by `"****"` — safe to return in API list/get responses without leaking secrets.
//!
//! # Encryption format
//!
//! Stored as `[24-byte XChaCha20 nonce || authenticated ciphertext]` (same format
//! as TOTP secrets in `mfa_totp_service`).

use crate::{DbConnection, ProviderConfiguration, ProviderConfigurationReposiory, ServiceError};

use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce,
};

use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

const NONCE_SIZE: usize = 24;

#[derive(diesel::AsChangeset)]
#[diesel(table_name = crate::schema::provider_configurations)]
struct ProviderConfigUpdateChangeset {
    config_encrypted: Vec<u8>,
    config_redacted: serde_json::Value,
    is_active: bool,
    updated_at: chrono::DateTime<chrono::Utc>,
    updated_by: Option<Uuid>,
}

pub struct ProviderConfigurationService {
    encryption_key: [u8; 32],
    provider_config_repo: Arc<ProviderConfigurationReposiory>,
}

impl ProviderConfigurationService {
    pub fn new(
        provider_config_repo: Arc<ProviderConfigurationReposiory>,
        encryption_key: [u8; 32],
    ) -> Self {
        Self {
            encryption_key,
            provider_config_repo,
        }
    }

    pub async fn create_provider_config(
        &self,
        conn: &mut DbConnection<'_>,
        config: &ProviderConfiguration,
    ) -> Result<ProviderConfiguration, ServiceError> {
        self.provider_config_repo
            .create(conn, config)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Encrypts provider configuration JSON bytes using XChaCha20-Poly1305.
    ///
    /// Output format (stored in DB as BYTEA):
    ///
    /// [ 24 bytes nonce | ciphertext ]
    ///
    /// # Security
    /// - Key must be 32 bytes
    /// - Nonce is randomly generated per encryption
    ///
    /// # Usage
    /// ```rust,ignore
    /// let encrypted = encrypt_config(config)?;
    /// ```
    pub fn encrypt_config(&self, config: &Value) -> Result<Vec<u8>, ServiceError> {
        let config_bytes = serde_json::to_vec(config)?;

        let cipher = XChaCha20Poly1305::new_from_slice(&self.encryption_key)
            .map_err(|_| ServiceError::InternalError("Invalid encryption key".to_string()))?;

        // Generate random nonce
        let nonce: XNonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, config_bytes.as_slice())
            .map_err(|e| ServiceError::InternalError(e.to_string()))?;

        // convert AFTER encrypt
        let nonce_bytes: [u8; NONCE_SIZE] = nonce.into();

        // Prepend nonce to ciphertext
        let mut output = Vec::with_capacity(nonce.len() + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);

        Ok(output)
    }

    /// Decrypts provider configuration bytes using XChaCha20-Poly1305.
    ///
    /// Input format (stored in DB as BYTEA): [ 24 bytes nonce | ciphertext ]
    pub fn decrypt_config(&self, encrypted: &[u8]) -> Result<Value, ServiceError> {
        if encrypted.len() < NONCE_SIZE {
            return Err(ServiceError::InternalError(
                "Invalid encrypted config: too short".to_string(),
            ));
        }
        let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
        let nonce = XNonce::from_slice(nonce_bytes);
        let cipher = XChaCha20Poly1305::new_from_slice(&self.encryption_key)
            .map_err(|_| ServiceError::InternalError("Invalid encryption key".to_string()))?;
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ServiceError::InternalError(e.to_string()))?;
        serde_json::from_slice(&plaintext).map_err(|e| ServiceError::InternalError(e.to_string()))
    }

    pub async fn has_active_email_provider(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<bool, ServiceError> {
        self.provider_config_repo
            .has_active_provider(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Recursively replace every leaf value in a JSON object with `"****"`.
    ///
    /// The *key names* are preserved (so API consumers can see `"api_key"` vs
    /// `"password"` in the response and know which auth mode is configured),
    /// but their values are masked.  Arrays are redacted element-by-element.
    pub fn redact_config(&self, config: &Value) -> Value {
        match config {
            Value::Object(map) => {
                let mut out = serde_json::Map::with_capacity(map.len());
                for (k, v) in map {
                    out.insert(k.clone(), self.redact_config(v));
                }
                Value::Object(out)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(|v| self.redact_config(v)).collect()),
            _ => Value::String("****".to_string()),
        }
    }

    pub async fn list_configs(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<Vec<ProviderConfiguration>, ServiceError> {
        self.provider_config_repo
            .list_all(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn update_config(
        &self,
        conn: &mut DbConnection<'_>,
        config_id: Uuid,
        config: &Value,
        is_active: bool,
        updated_by: Option<Uuid>,
    ) -> Result<ProviderConfiguration, ServiceError> {
        let config_encrypted = self.encrypt_config(config)?;
        let config_redacted = self.redact_config(config);
        let changeset = ProviderConfigUpdateChangeset {
            config_encrypted,
            config_redacted,
            is_active,
            updated_at: chrono::Utc::now(),
            updated_by,
        };
        self.provider_config_repo
            .update(conn, config_id, changeset)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    ServiceError::NotFoundError("Provider configuration not found".into())
                }
                _ => ServiceError::DatabaseError(e.to_string()),
            })
    }

    pub async fn find_config(
        &self,
        conn: &mut DbConnection<'_>,
        config_id: Uuid,
    ) -> Result<Option<ProviderConfiguration>, ServiceError> {
        self.provider_config_repo
            .find_by_id(conn, config_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn delete_config(
        &self,
        conn: &mut DbConnection<'_>,
        config_id: Uuid,
    ) -> Result<(), ServiceError> {
        self.provider_config_repo
            .delete(conn, config_id)
            .await
            .map(|_| ())
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    ServiceError::NotFoundError("Provider configuration not found".into())
                }
                _ => ServiceError::DatabaseError(e.to_string()),
            })
    }
}
