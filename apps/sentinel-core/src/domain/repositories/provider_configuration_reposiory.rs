//! Repository for the `provider_configurations` table.
//!
//! Stores SMTP provider configurations.  `config_encrypted` contains the XChaCha20-Poly1305
//! ciphertext of the full config JSON (including secrets); `config_redacted` contains a safe
//! copy where all secret values are replaced with `"****"`.
//!
//! The macro provides all needed CRUD — no custom methods are required.

use crate::impl_repository;
use crate::ProviderConfiguration;

use uuid::Uuid;

impl_repository!(
    ProviderConfigurationReposiory for ProviderConfiguration,
    crate::schema::provider_configurations::table,
    crate::schema::provider_configurations::configuration_id,
    Uuid
);

impl ProviderConfigurationReposiory {
    pub async fn has_active_provider(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<bool, crate::RepositoryError> {
        use crate::schema::provider_configurations::dsl::{is_active, provider_configurations};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        provider_configurations
            .filter(is_active.eq(true))
            .first::<crate::ProviderConfiguration>(conn)
            .await
            .optional()
            .map(|r| r.is_some())
            .map_err(crate::RepositoryError::Database)
    }

    pub async fn list_all(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<Vec<crate::ProviderConfiguration>, crate::RepositoryError> {
        use crate::schema::provider_configurations::dsl::provider_configurations;
        use diesel_async::RunQueryDsl;
        provider_configurations
            .load::<crate::ProviderConfiguration>(conn)
            .await
            .map_err(crate::RepositoryError::Database)
    }
}
