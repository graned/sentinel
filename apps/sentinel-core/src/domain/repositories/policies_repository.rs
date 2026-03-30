//! Repository for the `policies` table.
//!
//! Each policy is a named RBAC ruleset that can have multiple immutable versions.
//! `active_version` points to the currently enforced version number.
//! Custom methods (`find_first`, `find_all`, `deactivate`) support the
//! policy management UI and the in-memory engine hot-reload logic.

use crate::impl_repository;
use crate::Policy;

use uuid::Uuid;

impl_repository!(
    PolicyRepository for Policy,
    crate::schema::policies::table,
    crate::schema::policies::policy_id,
    Uuid
);

impl PolicyRepository {
    /// Returns the first active policy ordered by `created_at` ascending.
    /// Used when no explicit `policy_id` is provided on authorization checks.
    pub async fn find_first(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<Option<Policy>, diesel::result::Error> {
        use diesel::prelude::*;
        use diesel::OptionalExtension;
        use diesel_async::RunQueryDsl;

        crate::schema::policies::table
            .filter(crate::schema::policies::is_active.eq(true))
            .order(crate::schema::policies::created_at.asc())
            .first::<Policy>(conn)
            .await
            .optional()
    }

    /// Returns all active policies ordered by `created_at` ascending.
    pub async fn find_all(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<Vec<Policy>, diesel::result::Error> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        crate::schema::policies::table
            .filter(crate::schema::policies::is_active.eq(true))
            .order(crate::schema::policies::created_at.asc())
            .load::<Policy>(conn)
            .await
    }

    /// Soft-deactivate a policy by setting `is_active = false`.
    pub async fn deactivate(
        &self,
        conn: &mut crate::DbConnection<'_>,
        id: Uuid,
    ) -> Result<Policy, diesel::result::Error> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        diesel::update(crate::schema::policies::table.find(id))
            .set(crate::schema::policies::is_active.eq(false))
            .get_result::<Policy>(conn)
            .await
    }
}
