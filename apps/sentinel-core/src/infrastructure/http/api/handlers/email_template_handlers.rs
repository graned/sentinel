//! HTTP handlers for admin email template management (`/v1/api/admin/email-templates`).
//!
//! Email templates allow admins to customize the subject, plain-text body, and optional
//! HTML body of transactional emails (verification, password reset, password change).
//! When no active custom template exists for a type, the system falls back to built-in
//! defaults defined in `email_template_service::builtin_default`.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET    | `/admin/email-templates` | List all templates (active and inactive) |
//! | POST   | `/admin/email-templates` | Create and activate a new template |
//! | PUT    | `/admin/email-templates/{id}` | Update an existing template |

use crate::{
    http::api::dtos::{
        AuthenticatedUserContext, CreateEmailTemplateRequest, EmailTemplateResponse,
        UpdateEmailTemplateRequest,
    },
    http::api::routes::api_validation::ValidatedJson,
    http::api::RawResponse,
    http::server::AppState,
    ApiError,
};
use axum::{extract::Path, Extension};
use std::sync::Arc;
use uuid::Uuid;

/// GET /v1/api/admin/email-templates
/// List all email templates. Requires admin role.
#[utoipa::path(
    get,
    path = "/v1/api/admin/email-templates",
    responses(
        (status = 200, description = "List of email templates", body = Vec<EmailTemplateResponse>),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn list_email_templates(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
) -> Result<RawResponse<Vec<EmailTemplateResponse>>, ApiError> {
    state
        .email_template_application
        .list_templates(&ctx)
        .await
        .map(|templates| RawResponse(templates.into_iter().map(EmailTemplateResponse::from).collect()))
        .map_err(ApiError::from)
}

/// POST /v1/api/admin/email-templates
/// Create a new email template. Requires admin role.
#[utoipa::path(
    post,
    path = "/v1/api/admin/email-templates",
    request_body = CreateEmailTemplateRequest,
    responses(
        (status = 200, description = "Template created", body = EmailTemplateResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn create_email_template(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    ValidatedJson(req): ValidatedJson<CreateEmailTemplateRequest>,
) -> Result<RawResponse<EmailTemplateResponse>, ApiError> {
    state
        .email_template_application
        .create_template(&ctx, req.template_type, req.subject, req.body_text, req.body_html)
        .await
        .map(|t| RawResponse(EmailTemplateResponse::from(t)))
        .map_err(ApiError::from)
}

/// PUT /v1/api/admin/email-templates/{template_id}
/// Update an existing email template. Requires admin role.
#[utoipa::path(
    put,
    path = "/v1/api/admin/email-templates/{template_id}",
    params(
        ("template_id" = Uuid, Path, description = "UUID of the template to update"),
    ),
    request_body = UpdateEmailTemplateRequest,
    responses(
        (status = 200, description = "Template updated", body = EmailTemplateResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "Template not found"),
    ),
    security(("BearerAuth" = [])),
    tag = "admin"
)]
pub async fn update_email_template(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ctx): Extension<AuthenticatedUserContext>,
    Path(template_id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateEmailTemplateRequest>,
) -> Result<RawResponse<EmailTemplateResponse>, ApiError> {
    state
        .email_template_application
        .update_template(
            &ctx,
            template_id,
            req.subject,
            req.body_text,
            req.body_html,
            req.is_active,
        )
        .await
        .map(|t| RawResponse(EmailTemplateResponse::from(t)))
        .map_err(ApiError::from)
}
