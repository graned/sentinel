//! DTOs for admin email-template endpoints: create, update, and list.

use crate::EmailTemplateType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct CreateEmailTemplateRequest {
    pub template_type: EmailTemplateType,
    #[validate(length(min = 1, message = "Subject must not be empty"))]
    pub subject: String,
    #[validate(length(min = 1, message = "Body text must not be empty"))]
    pub body_text: String,
    pub body_html: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct UpdateEmailTemplateRequest {
    pub subject: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EmailTemplateResponse {
    pub template_id: Uuid,
    pub template_type: EmailTemplateType,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<crate::EmailTemplate> for EmailTemplateResponse {
    fn from(t: crate::EmailTemplate) -> Self {
        Self {
            template_id: t.template_id,
            template_type: t.template_type,
            subject: t.subject,
            body_text: t.body_text,
            body_html: t.body_html,
            is_active: t.is_active,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}
