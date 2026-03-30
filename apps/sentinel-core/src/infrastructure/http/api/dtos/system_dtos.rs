//! DTOs for system configuration endpoints: SMTP provider config (create, update,
//! list, reveal, test, send-test) and auth-methods discovery.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

/// Public information about a registered OIDC client.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OidcClientInfo {
    pub client_id: String,
    pub name: String,
    pub allowed_scopes: Vec<String>,
    pub pkce_required: bool,
}

/// System-level information about which authentication methods are active.
/// Returned by `GET /v1/api/auth/auth-methods` — no auth required.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AuthMethodsResponse {
    /// Password-based login is always supported.
    pub password_enabled: bool,
    /// TOTP MFA is available (per-user opt-in enrollment).
    pub mfa_totp_available: bool,
    /// Long-lived API tokens can be created (admin-gated).
    pub api_tokens_available: bool,
    /// New users must verify their email before accessing protected endpoints.
    pub email_verification_required: bool,
    /// Whether a live email provider (SMTP, etc.) is configured in the DB.
    pub email_provider_active: bool,
    /// Whether at least one OIDC client is registered.
    pub oidc_enabled: bool,
    /// All registered OIDC clients (public info only).
    pub oidc_clients: Vec<OidcClientInfo>,
}

/// Request payload for creating or updating a provider configuration.
///
/// This endpoint allows admins to register email provider settings
/// (SMTP, SES, Mailgun, etc.) that will be used by the system to send emails.
///
/// ## Security
/// - `config` MAY contain secrets (passwords, API keys).
/// - Secrets are encrypted before being stored.
/// - Secrets are NEVER returned in API responses.
/// - Clients should send full config on create/update.
///
/// ## Multi-Tenancy
/// - `tenant_id` can be omitted if derived from auth context.
/// - In a single-tenant setup, this can be `None`.
///
/// ## Example (SMTP)
/// ```json
/// {
///   "provider": "smtp",
///   "config": {
///     "host": "smtp.postmarkapp.com",
///     "port": 587,
///     "username": "abc123",
///     "password": "super-secret",
///     "use_tls": true,
///     "from_email": "no-reply@acme.com"
///   },
///   "is_active": true
/// }
/// ```
///
/// ## Example (Mailgun API)
/// ```json
/// {
///   "provider": "mailgun",
///   "config": {
///     "domain": "mg.acme.com",
///     "api_key": "key-xxxx",
///     "base_url": "https://api.eu.mailgun.net"
///   },
///   "is_active": true
/// }
/// ```
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateProviderConfigRequest {
    /// Optional tenant identifier.
    /// If omitted, the system may infer it from the authenticated user.
    pub tenant_id: Option<Uuid>,

    /// Provider identifier.
    ///
    /// Common values:
    /// - "smtp"
    /// - "ses"
    /// - "mailgun"
    /// - "postmark"
    pub provider: String,

    /// Provider-specific configuration.
    ///
    /// This may include sensitive fields like:
    /// - password
    /// - api_key
    /// - token
    ///
    /// These values will be encrypted at rest.
    #[schema(value_type = Object)]
    pub config: Value,

    /// Whether this configuration should be set as active.
    ///
    /// Only one active config per provider per tenant is allowed.
    #[serde(default = "default_true")]
    pub is_active: bool,
}

fn default_true() -> bool {
    true
}

/// Response object returned after creating or fetching
/// a provider configuration.
///
/// ## Security
/// - Secrets are NEVER returned.
/// - `config_redacted` contains masked values.
///
/// ## Example Response
/// ```json
/// {
///   "configuration_id": "c9f2c9c8-8c6a-4d1c-b8c5-d9e5f4e8f111",
///   "tenant_id": null,
///   "provider": "smtp",
///   "config_redacted": {
///     "host": "smtp.postmarkapp.com",
///     "port": 587,
///     "username": "abc123",
///     "password": "********",
///     "use_tls": true
///   },
///   "is_active": true,
///   "last_tested_at": "2026-01-30T12:00:00Z",
///   "last_test_success": true,
///   "created_at": "2026-01-30T11:58:00Z",
///   "updated_at": "2026-01-30T12:00:00Z"
/// }
/// ```
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProviderConfigResponse {
    pub configuration_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub provider: String,
    #[schema(value_type = Object)]
    pub config_redacted: Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to update an existing provider configuration.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateProviderConfigRequest {
    /// New provider-specific configuration. All values will be re-encrypted.
    #[schema(value_type = Object)]
    pub config: Value,

    /// Whether this configuration should be active.
    pub is_active: bool,
}

/// Result of an SMTP connection test.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TestProviderConfigResponse {
    /// `true` when the SMTP handshake succeeded.
    pub success: bool,
    /// Human-readable result message.
    pub message: String,
}

/// Request body for the send-test-email endpoint.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct SendTestEmailRequest {
    /// Recipient address for the test email.
    #[validate(email)]
    pub to_email: String,
}

/// Response containing decrypted provider configuration (admin reveal endpoint).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DecryptedProviderConfigResponse {
    pub configuration_id: Uuid,
    pub provider: String,
    /// Decrypted configuration — contains plaintext secrets. Handle with care.
    #[schema(value_type = Object)]
    pub config: Value,
    pub is_active: bool,
}
