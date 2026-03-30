//! Repository for the `email_templates` table.
//!
//! Stores admin-configurable email templates.  A partial unique index on
//! `(template_type) WHERE is_active = TRUE` enforces at most one active template
//! per type at the database level.  The `list_all` custom method returns all rows
//! regardless of `is_active`, for the admin management UI.

use crate::impl_repository;
use crate::EmailTemplate;
use uuid::Uuid;

impl_repository!(
    EmailTemplateRepository for EmailTemplate,
    crate::schema::email_templates::table,
    crate::schema::email_templates::template_id,
    Uuid
);

impl EmailTemplateRepository {
    pub async fn list_all(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<Vec<EmailTemplate>, diesel::result::Error> {
        use crate::schema::email_templates::table;
        use diesel_async::RunQueryDsl;

        table.load(conn).await
    }
}
