//! Repository for the `oidc_signing_keys` table.
//!
//! Stores RSA-2048 signing keys (encrypted private key DER + public JWK JSON).
//! At most one key has `status = "active"` at any time; generating a new key
//! retires all previously active ones via `retire_all_active`.

use crate::impl_repository;
use crate::OidcSigningKey;

use uuid::Uuid;

impl_repository!(
    OidcSigningKeyRepository for OidcSigningKey,
    crate::schema::oidc_signing_keys::table,
    crate::schema::oidc_signing_keys::oidc_signing_key_id,
    Uuid
);

impl OidcSigningKeyRepository {
    /// Return the most-recently-created key with `status = "active"`, or `None`.
    pub async fn find_active(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<Option<OidcSigningKey>, crate::RepositoryError> {
        use crate::schema::oidc_signing_keys::dsl::{
            created_at as col_created_at, oidc_signing_keys, status as col_status,
        };
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        oidc_signing_keys
            .filter(col_status.eq("active"))
            .order(col_created_at.desc())
            .first::<OidcSigningKey>(conn)
            .await
            .optional()
            .map_err(crate::RepositoryError::Database)
    }

    /// Set `status = "retired"` on all currently active keys.
    /// Called before inserting a new key to maintain the single-active-key invariant.
    pub async fn retire_all_active(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<usize, crate::RepositoryError> {
        use crate::schema::oidc_signing_keys::dsl::{oidc_signing_keys, status as col_status};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        diesel::update(oidc_signing_keys.filter(col_status.eq("active")))
            .set(col_status.eq("retired"))
            .execute(conn)
            .await
            .map_err(crate::RepositoryError::Database)
    }
}
