//! Repository for the `oidc_auth_codes` table.
//!
//! Authorization codes are short-lived (2 minutes) and single-use.  The critical
//! `consume_code` method performs an atomic UPDATE that simultaneously checks
//! `consumed_at IS NULL` and `expires_at > now()` — preventing replay attacks
//! without a separate SELECT + UPDATE round trip.

use crate::impl_repository;
use crate::OidcAuthCode;

use uuid::Uuid;

impl_repository!(
    OidcAuthCodeRepository for OidcAuthCode,
    crate::schema::oidc_auth_codes::table,
    crate::schema::oidc_auth_codes::oidc_auth_code_id,
    Uuid
);

impl OidcAuthCodeRepository {
    pub async fn find_by_code_hash(
        &self,
        conn: &mut crate::DbConnection<'_>,
        code_hash: &str,
    ) -> Result<Option<OidcAuthCode>, crate::RepositoryError> {
        use crate::schema::oidc_auth_codes::dsl::{code_hash as col_code_hash, oidc_auth_codes};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        oidc_auth_codes
            .filter(col_code_hash.eq(code_hash))
            .first::<OidcAuthCode>(conn)
            .await
            .optional()
            .map_err(crate::RepositoryError::Database)
    }

    /// Atomically mark an auth code as consumed.
    ///
    /// Applies an UPDATE with three conditions: matching hash, not yet consumed, and not
    /// expired.  Returns `RepositoryError::NotFound` if any condition fails so callers
    /// cannot distinguish "wrong hash" from "already used" or "expired".
    pub async fn consume_code(
        &self,
        conn: &mut crate::DbConnection<'_>,
        code_hash: &str,
    ) -> Result<OidcAuthCode, crate::RepositoryError> {
        use crate::schema::oidc_auth_codes::dsl::{
            code_hash as col_code_hash, consumed_at as col_consumed_at, expires_at as col_expires_at,
            oidc_auth_codes,
        };
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        diesel::update(
            oidc_auth_codes
                .filter(col_code_hash.eq(code_hash))
                .filter(col_consumed_at.is_null())
                .filter(col_expires_at.gt(chrono::Utc::now())),
        )
        .set(col_consumed_at.eq(chrono::Utc::now()))
        .get_result::<OidcAuthCode>(conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => crate::RepositoryError::NotFound,
            other => crate::RepositoryError::Database(other),
        })
    }
}
