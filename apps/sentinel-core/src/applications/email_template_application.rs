//! Email template application — admin CRUD for email templates with role guard.
//!
//! Thin wrapper around `EmailTemplateService` that:
//! 1. Enforces the `admin` role before any mutating operation.
//! 2. Translates DTOs to service request structs.
//!
//! Templates are typed (`EmailVerification`, `PasswordReset`, `PasswordChanged`) and
//! are used by `EmailService` to render the outgoing emails.  When no active custom
//! template exists for a type, `EmailTemplateService::render` falls back to built-in defaults.

use crate::{
    http::api::dtos::AuthenticatedUserContext, CreateEmailTemplateServiceRequest, EmailTemplate,
    EmailTemplateService, EmailTemplateType, PostgresClient, ServiceError,
    UpdateEmailTemplateServiceRequest,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct EmailTemplateApplication {
    email_template_service: Arc<EmailTemplateService>,
    pg_client: Arc<PostgresClient>,
}

impl EmailTemplateApplication {
    pub fn new(
        email_template_service: Arc<EmailTemplateService>,
        pg_client: Arc<PostgresClient>,
    ) -> Self {
        Self {
            email_template_service,
            pg_client,
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

    pub async fn create_template(
        &self,
        ctx: &AuthenticatedUserContext,
        template_type: EmailTemplateType,
        subject: String,
        body_text: String,
        body_html: Option<String>,
    ) -> Result<EmailTemplate, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        self.email_template_service
            .create_template(
                &mut conn,
                CreateEmailTemplateServiceRequest {
                    template_type,
                    subject,
                    body_text,
                    body_html,
                    created_by: Some(ctx.user_id),
                },
            )
            .await
    }

    pub async fn update_template(
        &self,
        ctx: &AuthenticatedUserContext,
        template_id: Uuid,
        subject: Option<String>,
        body_text: Option<String>,
        body_html: Option<String>,
        is_active: Option<bool>,
    ) -> Result<EmailTemplate, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        self.email_template_service
            .update_template(
                &mut conn,
                template_id,
                UpdateEmailTemplateServiceRequest {
                    subject,
                    body_text,
                    body_html,
                    is_active,
                },
                Some(ctx.user_id),
            )
            .await
    }

    pub async fn list_templates(
        &self,
        ctx: &AuthenticatedUserContext,
    ) -> Result<Vec<EmailTemplate>, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        self.email_template_service.list_templates(&mut conn).await
    }
}
