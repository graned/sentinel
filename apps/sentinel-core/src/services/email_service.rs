//! SMTP email delivery service — provider selection, decryption, and lettre transport.
//!
//! # How it works
//!
//! 1. On each send call, the service queries `provider_configurations` for the active
//!    SMTP config (there should be at most one active config at a time).
//! 2. It decrypts the config using [`ProviderConfigurationService::decrypt_config`].
//! 3. It builds a `lettre` STARTTLS transport from the decrypted credentials.
//! 4. It renders the appropriate email template (custom or built-in default) via
//!    [`EmailTemplateService::render`], populating `{{placeholder}}` tokens.
//! 5. It sends the email.
//!
//! # No SMTP configured
//!
//! If no active provider config exists, send methods log a warning and return `Ok(())`
//! so registration and password-reset flows always succeed regardless of email setup.
//!
//! # Auth mode support
//!
//! The `build_mailer` helper accepts both `password` and `api_key` fields as the
//! credential secret so services like Resend (username=`"resend"`, api_key=`<key>`)
//! work without special-casing in the config schema.

use crate::{
    DbConnection, EmailTemplateService, EmailTemplateType, ProviderConfigurationReposiory,
    ProviderConfigurationService, ServiceError,
};
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::collections::HashMap;
use std::sync::Arc;

pub struct EmailService {
    provider_config_repo: Arc<ProviderConfigurationReposiory>,
    provider_config_service: Arc<ProviderConfigurationService>,
    email_template_service: Arc<EmailTemplateService>,
    frontend_url: String,
}

impl EmailService {
    pub fn new(
        provider_config_repo: Arc<ProviderConfigurationReposiory>,
        provider_config_service: Arc<ProviderConfigurationService>,
        email_template_service: Arc<EmailTemplateService>,
        frontend_url: String,
    ) -> Self {
        Self {
            provider_config_repo,
            provider_config_service,
            email_template_service,
            frontend_url,
        }
    }

    pub fn frontend_url(&self) -> &str {
        &self.frontend_url
    }

    /// Builds an authenticated SMTP transport from a decrypted provider config.
    ///
    /// Handles both credential-based (`password`) and API-key-based (`api_key`)
    /// providers (e.g. Resend: username=`"resend"`, api_key=`<key>`).
    ///
    /// Authentication is explicitly set to `[Plain, Login]` so providers that
    /// only advertise `AUTH PLAIN` (Resend, Mailjet, etc.) work reliably without
    /// depending on the server's EHLO negotiation order.
    fn build_mailer(
        config: &serde_json::Value,
    ) -> Result<AsyncSmtpTransport<Tokio1Executor>, ServiceError> {
        let host = config["host"]
            .as_str()
            .ok_or_else(|| {
                tracing::error!("build_mailer: SMTP config is missing 'host'");
                ServiceError::InternalError("SMTP config missing 'host'".to_string())
            })?
            .to_string();
        let port = config["port"].as_u64().ok_or_else(|| {
            tracing::error!("build_mailer: SMTP config is missing 'port'");
            ServiceError::InternalError("SMTP config missing 'port'".to_string())
        })? as u16;

        // username is required when api_key is the auth method (e.g. "resend" for Resend).
        let username = config["username"].as_str().unwrap_or("").to_string();

        // Accept either 'password' or 'api_key' as the credential secret.
        let has_api_key = config["api_key"].as_str().is_some();
        let secret = config["password"]
            .as_str()
            .or_else(|| config["api_key"].as_str())
            .ok_or_else(|| {
                tracing::error!(
                    "build_mailer: SMTP config is missing both 'password' and 'api_key'"
                );
                ServiceError::InternalError(
                    "SMTP config missing 'password' or 'api_key'".to_string(),
                )
            })?
            .to_string();

        if has_api_key && username.is_empty() {
            tracing::error!(
                "build_mailer: SMTP config uses api_key auth but 'username' is empty \
                 (e.g. set username to 'resend' for Resend SMTP)"
            );
            return Err(ServiceError::InternalError(
                "SMTP config uses api_key but 'username' is missing \
                 (e.g. set username to 'resend' for Resend SMTP)"
                    .to_string(),
            ));
        }

        let use_tls = config["use_tls"].as_bool().unwrap_or(false);
        tracing::debug!(
            host,
            port,
            username,
            use_tls,
            auth_method = if has_api_key { "api_key" } else { "password" },
            "build_mailer: building SMTP transport"
        );

        let creds = Credentials::new(username, secret);

        // Explicitly set PLAIN + LOGIN so API-key providers (Resend, Mailjet, etc.)
        // that only advertise AUTH PLAIN work reliably.
        let auth_mechanisms = vec![Mechanism::Plain, Mechanism::Login];

        let mailer = if use_tls {
            tracing::debug!("build_mailer: using TLS relay (port {})", port);
            AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
                .map_err(|e| {
                    tracing::error!(host, port, error = %e, "build_mailer: failed to create TLS relay transport");
                    ServiceError::InternalError(format!("SMTP relay error: {e}"))
                })?
                .port(port)
                .credentials(creds)
                .authentication(auth_mechanisms)
                .build()
        } else {
            tracing::debug!("build_mailer: using STARTTLS relay (port {})", port);
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
                .map_err(|e| {
                    tracing::error!(host, port, error = %e, "build_mailer: failed to create STARTTLS relay transport");
                    ServiceError::InternalError(format!("SMTP relay error: {e}"))
                })?
                .port(port)
                .credentials(creds)
                .authentication(auth_mechanisms)
                .build()
        };

        tracing::debug!(
            host,
            port,
            "build_mailer: SMTP transport built successfully"
        );
        Ok(mailer)
    }

    /// Internal helper: fetch active SMTP config, render template, and send email.
    async fn send_with_template(
        &self,
        conn: &mut DbConnection<'_>,
        to_email: &str,
        template_type: EmailTemplateType,
        context: HashMap<&str, &str>,
    ) -> Result<(), ServiceError> {
        use crate::schema::provider_configurations::is_active;
        use diesel::ExpressionMethods;

        tracing::debug!(
            to_email,
            template_type = ?template_type,
            "send_with_template: starting email send"
        );

        // 1. Query active provider config
        tracing::debug!("send_with_template: querying active provider config");
        let rows = self
            .provider_config_repo
            .find_where(conn, is_active.eq(true))
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "send_with_template: database error fetching provider config");
                ServiceError::DatabaseError(e.to_string())
            })?;

        if rows.is_empty() {
            tracing::warn!("send_with_template: no active SMTP provider configured — skipping email send to {to_email}");
            return Ok(());
        }
        tracing::debug!(
            config_id = %rows[0].configuration_id,
            "send_with_template: found active provider config"
        );

        // 2. Decrypt config
        tracing::debug!("send_with_template: decrypting provider config");
        let config = self
            .provider_config_service
            .decrypt_config(&rows[0].config_encrypted)
            .map_err(|e| {
                tracing::error!(
                    config_id = %rows[0].configuration_id,
                    error = ?e,
                    "send_with_template: failed to decrypt provider config"
                );
                e
            })?;

        // 3. Extract from_email for the message builder
        let from_email = config["from_email"]
            .as_str()
            .ok_or_else(|| {
                tracing::error!(
                    config_id = %rows[0].configuration_id,
                    "send_with_template: decrypted config is missing 'from_email'"
                );
                ServiceError::InternalError("SMTP config missing 'from_email'".to_string())
            })?
            .to_string();
        tracing::debug!(from_email, "send_with_template: using from_email");

        // 4. Render template (with fallback to built-in defaults)
        tracing::debug!(template_type = ?template_type.clone(), "send_with_template: rendering email template");
        let rendered = self
            .email_template_service
            .render(conn, template_type.clone(), &context)
            .await
            .map_err(|e| {
                tracing::error!(template_type = ?template_type.clone(), error = ?e, "send_with_template: failed to render email template");
                e
            })?;
        tracing::debug!(subject = %rendered.subject, has_html = rendered.body_html.is_some(), "send_with_template: template rendered");

        // 5. Build email message (plain text; use body_html if present)
        tracing::debug!("send_with_template: building email message");
        let builder = Message::builder()
            .from(
                from_email
                    .parse()
                    .map_err(|e: lettre::address::AddressError| {
                        tracing::error!(from_email, error = %e, "send_with_template: 'from_email' is not a valid email address");
                        ServiceError::InternalError(format!("Invalid from_email: {e}"))
                    })?,
            )
            .to(to_email
                .parse()
                .map_err(|e: lettre::address::AddressError| {
                    tracing::error!(to_email, error = %e, "send_with_template: 'to_email' is not a valid email address");
                    ServiceError::InternalError(format!("Invalid to_email: {e}"))
                })?)
            .subject(rendered.subject);

        let email = if let Some(html) = rendered.body_html {
            builder
                .header(ContentType::TEXT_HTML)
                .body(html)
                .map_err(|e| {
                    tracing::error!(error = %e, "send_with_template: failed to build HTML email message");
                    ServiceError::InternalError(format!("Failed to build email: {e}"))
                })?
        } else {
            builder
                .header(ContentType::TEXT_PLAIN)
                .body(rendered.body_text)
                .map_err(|e| {
                    tracing::error!(error = %e, "send_with_template: failed to build plain-text email message");
                    ServiceError::InternalError(format!("Failed to build email: {e}"))
                })?
        };

        // 6. Build mailer (with explicit auth mechanisms) and send
        tracing::debug!("send_with_template: building SMTP transport");
        let mailer = Self::build_mailer(&config)?;

        tracing::debug!(to_email, "send_with_template: sending email via SMTP");
        mailer.send(email).await.map_err(|e| {
            tracing::error!(to_email, error = %e, "send_with_template: SMTP send failed");
            ServiceError::InternalError(format!("Failed to send email: {e}"))
        })?;

        tracing::info!(
            to_email,
            template_type = ?template_type,
            "send_with_template: email sent successfully"
        );
        Ok(())
    }

    /// Sends an email verification link. Silently skips if no SMTP is configured.
    pub async fn send_verification_email(
        &self,
        conn: &mut DbConnection<'_>,
        to_email: &str,
        first_name: &str,
        raw_token: &str,
    ) -> Result<(), ServiceError> {
        tracing::info!(
            to_email,
            "send_verification_email: sending verification email"
        );
        let link = format!("{}/verify-email?token={}", self.frontend_url, raw_token);
        tracing::debug!(verification_link = %link, "send_verification_email: verification link constructed");
        let mut ctx = HashMap::new();
        ctx.insert("first_name", first_name);
        ctx.insert("verification_link", link.as_str());
        self.send_with_template(conn, to_email, EmailTemplateType::EmailVerification, ctx)
            .await
    }

    /// Sends a password reset link. Silently skips if no SMTP is configured.
    pub async fn send_password_reset_email(
        &self,
        conn: &mut DbConnection<'_>,
        to_email: &str,
        first_name: &str,
        raw_token: &str,
    ) -> Result<(), ServiceError> {
        let link = format!("{}/reset-password?token={}", self.frontend_url, raw_token);
        let mut ctx = HashMap::new();
        ctx.insert("first_name", first_name);
        ctx.insert("reset_link", link.as_str());
        self.send_with_template(conn, to_email, EmailTemplateType::PasswordReset, ctx)
            .await
    }

    /// Sends a one-off plain-text test email using the given decrypted config.
    /// Does not use templates or the DB — uses `config` directly.
    pub async fn send_test_email(
        &self,
        config: &serde_json::Value,
        to_email: &str,
    ) -> Result<(), ServiceError> {
        let from_email = config["from_email"]
            .as_str()
            .ok_or_else(|| {
                ServiceError::InternalError("SMTP config missing 'from_email'".to_string())
            })?
            .to_string();

        let email = Message::builder()
            .from(
                from_email
                    .parse()
                    .map_err(|e: lettre::address::AddressError| {
                        ServiceError::InternalError(format!("Invalid from_email: {e}"))
                    })?,
            )
            .to(to_email
                .parse()
                .map_err(|e: lettre::address::AddressError| {
                    ServiceError::InternalError(format!("Invalid to_email: {e}"))
                })?)
            .subject("Sentinel Auth — Test Email")
            .header(ContentType::TEXT_PLAIN)
            .body(
                "This is a test email sent from Sentinel Auth to verify your SMTP configuration."
                    .to_string(),
            )
            .map_err(|e| ServiceError::InternalError(format!("Failed to build email: {e}")))?;

        let mailer = Self::build_mailer(config)?;
        mailer
            .send(email)
            .await
            .map_err(|e| ServiceError::InternalError(format!("Failed to send test email: {e}")))?;

        tracing::info!("Test email sent to {to_email}");
        Ok(())
    }

    /// Tests the SMTP connection for the given decrypted config value.
    /// Returns `Ok(())` on a successful handshake, `Err` with a human-readable
    /// message on failure.
    pub async fn test_connection(&self, config: &serde_json::Value) -> Result<(), ServiceError> {
        let mailer = Self::build_mailer(config)?;
        let ok = mailer.test_connection().await.map_err(|e| {
            ServiceError::InternalError(format!("SMTP connection test failed: {e}"))
        })?;

        if ok {
            Ok(())
        } else {
            Err(ServiceError::InternalError(
                "SMTP server accepted connection but reported not OK".to_string(),
            ))
        }
    }

    /// Sends a "password changed" notification. Silently skips if no SMTP is configured.
    pub async fn send_password_changed_email(
        &self,
        conn: &mut DbConnection<'_>,
        to_email: &str,
        first_name: &str,
    ) -> Result<(), ServiceError> {
        let mut ctx = HashMap::new();
        ctx.insert("first_name", first_name);
        self.send_with_template(conn, to_email, EmailTemplateType::PasswordChanged, ctx)
            .await
    }
}
