//! Repository for the `policy_versions` table.
//!
//! Each policy version is an immutable snapshot of compiled rules.  The `compiled_rules`
//! column stores the bincode-serialised trie produced by `sentinel_policy_engine::compile`.
//! Custom methods manage activation/deactivation and version lookup for the policy engine
//! hot-reload flow.

use crate::impl_repository;
use crate::PolicyVersion;
use diesel::OptionalExtension;
use diesel_async::RunQueryDsl;

use uuid::Uuid;

impl_repository!(
    PolicyVersionRepository for PolicyVersion,
    crate::schema::policy_versions::table,
    crate::schema::policy_versions::policy_version_id,
    Uuid
);

impl PolicyVersionRepository {
    pub async fn find_latest_for_policy_version(
        &self,
        conn: &mut DbConnection<'_>,
        pid: uuid::Uuid,
    ) -> Result<Option<PolicyVersion>, diesel::result::Error> {
        let q = diesel::sql_query(
            r#"
            SELECT *
            FROM policy_versions
            WHERE policy_id = $1
            ORDER BY version DESC
            LIMIT 1
            "#,
        )
        .bind::<diesel::sql_types::Uuid, _>(pid);

        // `optional()` turns NotFound into Ok(None)
        q.get_result::<PolicyVersion>(conn).await.optional()
    }

    /// Fetch a specific version by policy_id + version number
    pub async fn find_by_policy_and_version(
        &self,
        conn: &mut DbConnection<'_>,
        pid: Uuid,
        version: i64,
    ) -> Result<Option<PolicyVersion>, diesel::result::Error> {
        let q = diesel::sql_query(
            r#"
            SELECT *
            FROM policy_versions
            WHERE policy_id = $1
              AND version = $2
            LIMIT 1
            "#,
        )
        .bind::<diesel::sql_types::Uuid, _>(pid)
        .bind::<diesel::sql_types::BigInt, _>(version);

        q.get_result::<PolicyVersion>(conn).await.optional()
    }

    /// Mark all versions of a policy as inactive.
    /// Called before activating a new version so only one is ever active.
    pub async fn deactivate_all_for_policy(
        &self,
        conn: &mut crate::DbConnection<'_>,
        pid: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        use diesel::{ExpressionMethods, QueryDsl};
        use diesel_async::RunQueryDsl;

        diesel::update(
            crate::schema::policy_versions::table
                .filter(crate::schema::policy_versions::policy_id.eq(pid)),
        )
        .set(crate::schema::policy_versions::is_active.eq(false))
        .execute(conn)
        .await
    }

    /// Mark a specific version as active.
    pub async fn set_version_active(
        &self,
        conn: &mut crate::DbConnection<'_>,
        pid: Uuid,
        version: i64,
    ) -> Result<usize, diesel::result::Error> {
        use diesel::{ExpressionMethods, QueryDsl};
        use diesel_async::RunQueryDsl;

        diesel::update(
            crate::schema::policy_versions::table
                .filter(crate::schema::policy_versions::policy_id.eq(pid))
                .filter(crate::schema::policy_versions::version.eq(version)),
        )
        .set(crate::schema::policy_versions::is_active.eq(true))
        .execute(conn)
        .await
    }
}
