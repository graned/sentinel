//! Email template rendering with `{{placeholder}}` substitution.
//!
//! Admins can configure custom templates via the API for each [`EmailTemplateType`].
//! When no active custom template exists for a type, the service falls back to
//! built-in defaults defined in [`builtin_default`] so the system works
//! out of the box without any configuration.
//!
//! # Placeholder format
//!
//! Placeholders use double-brace syntax: `{{key}}`. The `render_placeholders`
//! function iterates the `context` map and performs a simple `str::replace` for
//! each entry. Unknown placeholders (keys not in the context map) are left as-is.
//!
//! # Unique active template constraint
//!
//! The database enforces `UNIQUE (template_type) WHERE is_active = TRUE`.
//! [`EmailTemplateService::create_template`] deactivates any existing active
//! template of the same type *before* inserting the new one to uphold this constraint.

use crate::{DbConnection, EmailTemplate, EmailTemplateRepository, EmailTemplateType, ServiceError};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// A fully rendered email ready to send — subject and body with all placeholders substituted.
pub struct RenderedEmail {
    pub subject: String,
    pub body_text: String,
    /// HTML body, if the template defines one.
    pub body_html: Option<String>,
}

/// Manages email template storage and rendering.
pub struct EmailTemplateService {
    repo: Arc<EmailTemplateRepository>,
}

impl EmailTemplateService {
    pub fn new(repo: Arc<EmailTemplateRepository>) -> Self {
        Self { repo }
    }

    /// Find the active template for the given type, fall back to built-in defaults,
    /// and replace `{{key}}` placeholders from the context map.
    pub async fn render(
        &self,
        conn: &mut DbConnection<'_>,
        template_type: EmailTemplateType,
        context: &HashMap<&str, &str>,
    ) -> Result<RenderedEmail, ServiceError> {
        use crate::schema::email_templates::{is_active, template_type as col_template_type};
        use diesel::{BoolExpressionMethods, ExpressionMethods};

        let template_type_for_query = template_type.clone();
        let rows = self
            .repo
            .find_where(
                conn,
                col_template_type
                    .eq(template_type_for_query)
                    .and(is_active.eq(true)),
            )
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let (subject, body_text, body_html) = if let Some(tmpl) = rows.into_iter().next() {
            (tmpl.subject, tmpl.body_text, tmpl.body_html)
        } else {
            let (s, b) = builtin_default(&template_type);
            (s.to_string(), b.to_string(), None)
        };

        Ok(RenderedEmail {
            subject: render_placeholders(subject, context),
            body_text: render_placeholders(body_text, context),
            body_html: body_html.map(|h| render_placeholders(h, context)),
        })
    }

    /// Create a new template. Deactivates any existing active template of the same type first
    /// to uphold the unique partial index on (template_type) WHERE is_active = TRUE.
    pub async fn create_template(
        &self,
        conn: &mut DbConnection<'_>,
        req: CreateEmailTemplateServiceRequest,
    ) -> Result<EmailTemplate, ServiceError> {
        use crate::schema::email_templates::{is_active, template_type as col_template_type};
        use diesel::{BoolExpressionMethods, ExpressionMethods};

        // Deactivate existing active template of same type
        let template_type_clone = req.template_type.clone();
        let existing = self
            .repo
            .find_where(
                conn,
                col_template_type
                    .eq(template_type_clone)
                    .and(is_active.eq(true)),
            )
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        for tmpl in existing {
            #[derive(diesel::AsChangeset)]
            #[diesel(table_name = crate::schema::email_templates)]
            struct DeactivateChangeset {
                is_active: bool,
            }
            self.repo
                .update(conn, tmpl.template_id, DeactivateChangeset { is_active: false })
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        let now = chrono::Utc::now();
        let record = EmailTemplate {
            template_id: Uuid::new_v4(),
            template_type: req.template_type,
            subject: req.subject,
            body_text: req.body_text,
            body_html: req.body_html,
            is_active: true,
            created_at: now,
            updated_at: Some(now),
            created_by: req.created_by,
            updated_by: req.created_by,
        };

        self.repo
            .create(conn, &record)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Update fields on an existing template.
    pub async fn update_template(
        &self,
        conn: &mut DbConnection<'_>,
        template_id: Uuid,
        req: UpdateEmailTemplateServiceRequest,
        updated_by: Option<Uuid>,
    ) -> Result<EmailTemplate, ServiceError> {
        #[derive(diesel::AsChangeset)]
        #[diesel(table_name = crate::schema::email_templates)]
        struct UpdateChangeset {
            subject: Option<String>,
            body_text: Option<String>,
            body_html: Option<Option<String>>,
            is_active: Option<bool>,
            updated_at: Option<chrono::DateTime<chrono::Utc>>,
            updated_by: Option<Option<Uuid>>,
        }

        self.repo
            .update(
                conn,
                template_id,
                UpdateChangeset {
                    subject: req.subject,
                    body_text: req.body_text,
                    body_html: req.body_html.map(Some),
                    is_active: req.is_active,
                    updated_at: Some(chrono::Utc::now()),
                    updated_by: Some(updated_by),
                },
            )
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// List all email templates.
    pub async fn list_templates(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<Vec<EmailTemplate>, ServiceError> {
        self.repo
            .list_all(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }
}

pub struct CreateEmailTemplateServiceRequest {
    pub template_type: EmailTemplateType,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub created_by: Option<Uuid>,
}

pub struct UpdateEmailTemplateServiceRequest {
    pub subject: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub is_active: Option<bool>,
}

/// Replace all `{{key}}` occurrences in `text` with the corresponding values from `context`.
/// Placeholders with no matching key are left unchanged.
fn render_placeholders(mut text: String, context: &HashMap<&str, &str>) -> String {
    for (key, value) in context {
        let placeholder = format!("{{{{{key}}}}}");
        text = text.replace(&placeholder, value);
    }
    text
}

/// Return the built-in (subject, body_text) pair for a given template type.
/// These are used when no custom active template is configured.
fn builtin_default(template_type: &EmailTemplateType) -> (&'static str, &'static str) {
    match template_type {
        EmailTemplateType::EmailVerification => (
            "Verify your email address",
            "Hi {{first_name}},\n\nPlease verify your email: {{verification_link}}",
        ),
        EmailTemplateType::PasswordReset => (
            "Reset your password",
            "Hi {{first_name}},\n\nClick here to reset your password: {{reset_link}}\n\nThis link expires in 1 hour. If you did not request this, ignore this email.",
        ),
        EmailTemplateType::PasswordChanged => (
            "Your password was changed",
            "Hi {{first_name}},\n\nYour password was recently changed. If this wasn't you, please contact support immediately.",
        ),
    }
}
