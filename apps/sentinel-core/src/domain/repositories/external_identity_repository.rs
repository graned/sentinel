//! Repository for the `external_identities` table.
//!
//! Provides CRUD operations and custom lookup methods for federated identity mappings.
//!
//! Custom methods beyond the macro-generated CRUD:
//! - [`find_by_provider_issuer_subject`](ExternalIdentityRepository::find_by_provider_issuer_subject) —
//!   primary lookup for the federation flow (multi-column unique constraint)
//! - [`update_last_login`](ExternalIdentityRepository::update_last_login) — timestamp update
//! - [`find_by_user_id`](ExternalIdentityRepository::find_by_user_id) — filtered list

use crate::impl_repository;
use crate::ExternalIdentity;
use uuid::Uuid;

// =============================================================================
// STANDARD CRUD: Use impl_repository! for all 13 auto-generated methods
// =============================================================================
//
// Per CLAUDE.md guidance:
// - "Repositories use an `impl_repository!` macro that provides 13 CRUD/pagination methods automatically"
// - "To add custom DB logic, add a separate `impl RepositoryName { ... }` block after the macro invocation"
// - "Do NOT import `DbConnection` at the top of a repository module file"
// =============================================================================

impl_repository!(
    ExternalIdentityRepository for ExternalIdentity,
    crate::schema::external_identities::table,
    crate::schema::external_identities::external_identity_id,
    Uuid
);

// =============================================================================
// CUSTOM METHODS: Only add methods the macro cannot handle
// =============================================================================
//
// Custom methods are needed for:
// 1. Multi-column lookups (provider + issuer + subject)
// 2. Bulk/timestamp updates not covered by macro's single-row update_where
// 3. Custom filtered lists (find by user_id)
// =============================================================================

impl ExternalIdentityRepository {
    /// Find an external identity by provider, issuer, and subject.
    /// This is the primary lookup for the federation flow.
    pub async fn find_by_provider_issuer_subject(
        &self,
        conn: &mut crate::DbConnection<'_>,
        provider: &str,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<ExternalIdentity>, crate::RepositoryError> {
        use crate::schema::external_identities::dsl::{
            external_identities, issuer as col_issuer, provider as col_provider,
            subject as col_subject,
        };
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let result = external_identities
            .filter(col_provider.eq(provider))
            .filter(col_issuer.eq(issuer))
            .filter(col_subject.eq(subject))
            .first::<ExternalIdentity>(conn)
            .await
            .optional()
            .map_err(crate::RepositoryError::from)?;

        Ok(result)
    }

    /// Update last_login_at for an external identity.
    pub async fn update_last_login(
        &self,
        conn: &mut crate::DbConnection<'_>,
        external_identity_id: Uuid,
    ) -> Result<(), crate::RepositoryError> {
        use crate::schema::external_identities::dsl::{
            external_identities, external_identity_id as col_id, last_login_at,
        };
        use diesel::{ExpressionMethods, QueryDsl};
        use diesel_async::RunQueryDsl;

        diesel::update(external_identities.filter(col_id.eq(external_identity_id)))
            .set(last_login_at.eq(Some(chrono::Utc::now())))
            .execute(conn)
            .await?;
        Ok(())
    }

    /// Find all external identities for a user.
    pub async fn find_by_user_id(
        &self,
        conn: &mut crate::DbConnection<'_>,
        user_id: Uuid,
    ) -> Result<Vec<ExternalIdentity>, crate::RepositoryError> {
        use crate::schema::external_identities::dsl::{
            external_identities, user_id as col_user_id,
        };
        use diesel::{ExpressionMethods, QueryDsl};

        let results = self.find_where(conn, col_user_id.eq(user_id)).await?;

        Ok(results)
    }
}
