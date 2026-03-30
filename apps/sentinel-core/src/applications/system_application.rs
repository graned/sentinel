//! System application — SMTP provider configuration, health check, and auth-method discovery.
//!
//! # SMTP provider configuration
//!
//! Admins can configure one or more SMTP providers (Resend, Mailjet, custom SMTP) via
//! this application.  Secrets are encrypted at rest; the stored `config_redacted` field
//! replaces all secret values with `"****"` and is safe to return in list/get responses.
//! The decrypted config is only returned on the explicit `/reveal` endpoint.
//!
//! # Methods
//!
//! - `add_provider_config` — create a new SMTP config (encrypts secrets)
//! - `list_configs` — list all configs (redacted)
//! - `update_config` — update fields; re-encrypts any changed secrets
//! - `delete_config` — hard-delete a config row
//! - `get_decrypted_config` — decrypt and return the full config (admin only)
//! - `test_config` — decrypt config and call `lettre::test_connection()`
//! - `send_test_email_config` — decrypt config and send a real test email
//! - `get_auth_methods` — return which auth methods are enabled (currently always `["password"]`)

use crate::{
    http::api::dtos::{
        AuthMethodsResponse, AuthenticatedUserContext, CreateProviderConfigRequest,
        DecryptedProviderConfigResponse, OidcClientInfo, ProviderConfigResponse,
        TestProviderConfigResponse, UpdateProviderConfigRequest,
    },
    EmailService, OidcClientService, PostgresClient, ProviderConfiguration,
    ProviderConfigurationService, ServiceError,
};
use chrono::Utc;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use std::sync::Arc;
use uuid::Uuid;

pub struct SystemApplication {
    pg_client: Arc<PostgresClient>,
    config_service: Arc<ProviderConfigurationService>,
    oidc_client_service: Arc<OidcClientService>,
    email_service: Arc<EmailService>,
}

impl SystemApplication {
    pub fn new(
        pg_client: Arc<PostgresClient>,
        config_service: Arc<ProviderConfigurationService>,
        oidc_client_service: Arc<OidcClientService>,
        email_service: Arc<EmailService>,
    ) -> Self {
        Self {
            pg_client,
            config_service,
            oidc_client_service,
            email_service,
        }
    }

    fn require_admin(ctx: &AuthenticatedUserContext) -> Result<(), ServiceError> {
        if !ctx.roles.iter().any(|r| r == "admin") {
            return Err(ServiceError::AuthorizationError(
                "Admin role required".into(),
            ));
        }
        Ok(())
    }

    fn config_to_response(c: ProviderConfiguration) -> ProviderConfigResponse {
        ProviderConfigResponse {
            configuration_id: c.configuration_id,
            tenant_id: c.tenant_id,
            provider: c.provider,
            config_redacted: c.config_redacted,
            is_active: c.is_active,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }

    pub async fn get_auth_methods(&self) -> Result<AuthMethodsResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        let email_provider_active = self
            .config_service
            .has_active_email_provider(&mut conn)
            .await?;
        let clients = self
            .oidc_client_service
            .list_all_clients(&mut conn)
            .await?;
        let oidc_enabled = !clients.is_empty();
        let oidc_clients = clients
            .into_iter()
            .map(|c| OidcClientInfo {
                client_id: c.client_id,
                name: c.name,
                allowed_scopes: c.allowed_scopes,
                pkce_required: c.pkce_required,
            })
            .collect();
        Ok(AuthMethodsResponse {
            password_enabled: true,
            mfa_totp_available: true,
            api_tokens_available: true,
            email_verification_required: true,
            email_provider_active,
            oidc_enabled,
            oidc_clients,
        })
    }

    pub async fn add_provider_config(
        &self,
        auth_context: AuthenticatedUserContext,
        request: CreateProviderConfigRequest,
    ) -> Result<ProviderConfigResponse, ServiceError> {
        tracing::debug!("Adding provider configuration {:#?}", request);
        let mut conn = self.pg_client.get_conn().await?;

        let config_service = self.config_service.clone();

        let response = conn
            .transaction(move |trx| {
                let config_service = config_service.clone();
                let mut request = request;

                async move {
                    // encrypt configuration
                    let config_encrypted = config_service.encrypt_config(&request.config)?;
                    tracing::debug!("Config encrypted!");
                    // create redacted config version
                    let config_redacted = config_service.redact_config(&mut request.config);
                    tracing::debug!("Config redacted! {:#?}", config_redacted);
                    let now = Utc::now();
                    // create config entity
                    let config = ProviderConfiguration {
                        configuration_id: Uuid::new_v4(),
                        tenant_id: request.tenant_id,
                        config_encrypted,
                        config_redacted: config_redacted.into(),
                        is_active: request.is_active,
                        provider: request.provider,
                        created_by: Some(auth_context.user_id),
                        created_at: now,
                        updated_by: Some(auth_context.user_id),
                        updated_at: now,
                    };
                    // check that configuration exists
                    // if config exists perform update otherwise create (upsert)
                    let new_config = config_service.create_provider_config(trx, &config).await?;
                    tracing::debug!(
                        "Provider {} config regisered successfully",
                        new_config.provider
                    );
                    Ok::<ProviderConfigResponse, ServiceError>(Self::config_to_response(new_config))
                }
                .scope_boxed()
            })
            .await?;
        Ok(response)
    }

    pub async fn list_configs(
        &self,
        ctx: &AuthenticatedUserContext,
    ) -> Result<Vec<ProviderConfigResponse>, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let configs = self.config_service.list_configs(&mut conn).await?;
        Ok(configs.into_iter().map(Self::config_to_response).collect())
    }

    pub async fn update_config(
        &self,
        ctx: &AuthenticatedUserContext,
        config_id: Uuid,
        req: UpdateProviderConfigRequest,
    ) -> Result<ProviderConfigResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let updated = self
            .config_service
            .update_config(&mut conn, config_id, &req.config, req.is_active, Some(ctx.user_id))
            .await?;
        Ok(Self::config_to_response(updated))
    }

    pub async fn delete_config(
        &self,
        ctx: &AuthenticatedUserContext,
        config_id: Uuid,
    ) -> Result<(), ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        // Verify exists before delete so we return a meaningful 404
        self.config_service
            .find_config(&mut conn, config_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Provider configuration not found".into()))?;
        self.config_service.delete_config(&mut conn, config_id).await
    }

    pub async fn test_config(
        &self,
        ctx: &AuthenticatedUserContext,
        config_id: Uuid,
    ) -> Result<TestProviderConfigResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let config = self
            .config_service
            .find_config(&mut conn, config_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Provider configuration not found".into()))?;
        let decrypted = self.config_service.decrypt_config(&config.config_encrypted)?;
        match self.email_service.test_connection(&decrypted).await {
            Ok(()) => Ok(TestProviderConfigResponse {
                success: true,
                message: "SMTP connection successful".to_string(),
            }),
            Err(ServiceError::InternalError(msg)) => Ok(TestProviderConfigResponse {
                success: false,
                message: msg,
            }),
            Err(e) => Err(e),
        }
    }

    pub async fn send_test_email_config(
        &self,
        ctx: &AuthenticatedUserContext,
        config_id: Uuid,
        to_email: &str,
    ) -> Result<TestProviderConfigResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let config = self
            .config_service
            .find_config(&mut conn, config_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Provider configuration not found".into()))?;
        let decrypted = self.config_service.decrypt_config(&config.config_encrypted)?;
        match self.email_service.send_test_email(&decrypted, to_email).await {
            Ok(()) => Ok(TestProviderConfigResponse {
                success: true,
                message: "Test email sent".to_string(),
            }),
            Err(ServiceError::InternalError(msg)) => Ok(TestProviderConfigResponse {
                success: false,
                message: msg,
            }),
            Err(e) => Err(e),
        }
    }

    pub async fn get_decrypted_config(
        &self,
        ctx: &AuthenticatedUserContext,
        config_id: Uuid,
    ) -> Result<DecryptedProviderConfigResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let config = self
            .config_service
            .find_config(&mut conn, config_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Provider configuration not found".into()))?;
        let decrypted = self
            .config_service
            .decrypt_config(&config.config_encrypted)?;
        Ok(DecryptedProviderConfigResponse {
            configuration_id: config.configuration_id,
            provider: config.provider,
            config: decrypted,
            is_active: config.is_active,
        })
    }
}
