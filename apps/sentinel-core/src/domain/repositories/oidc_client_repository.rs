//! Repository for the `oidc_clients` table.
//!
//! OIDC clients are registered external applications that use Sentinel as their
//! identity provider.  Lookups are typically by `client_id` (the public string
//! the relying party presents in OAuth flows), not by the internal UUID PK.

use crate::impl_repository;
use crate::OidcClient;

use uuid::Uuid;

impl_repository!(
    OidcClientRepository for OidcClient,
    crate::schema::oidc_clients::table,
    crate::schema::oidc_clients::oidc_client_id,
    Uuid
);

impl OidcClientRepository {
    pub async fn find_all(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<Vec<crate::OidcClient>, crate::RepositoryError> {
        use crate::schema::oidc_clients::dsl::oidc_clients;
        use diesel_async::RunQueryDsl;
        oidc_clients
            .load::<crate::OidcClient>(conn)
            .await
            .map_err(crate::RepositoryError::Database)
    }

    pub async fn find_by_client_id(
        &self,
        conn: &mut crate::DbConnection<'_>,
        client_id: &str,
    ) -> Result<Option<OidcClient>, crate::RepositoryError> {
        use crate::schema::oidc_clients::dsl::{client_id as col_client_id, oidc_clients};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        oidc_clients
            .filter(col_client_id.eq(client_id))
            .first::<OidcClient>(conn)
            .await
            .optional()
            .map_err(crate::RepositoryError::Database)
    }
}
