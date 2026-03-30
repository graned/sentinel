//! Repository for the `roles` table.
//!
//! Roles are flat string labels.  Built-in types (`user`, `admin`, `support`) are
//! seeded via migration; admins can create additional custom roles through the API.
//! The `list_all` custom method returns all roles ordered by `created_at` ascending.

use crate::impl_repository;
use crate::Role;

use uuid::Uuid;

impl_repository!(
    RoleRepository for Role,
    crate::schema::roles::table,
    crate::schema::roles::role_id,
    Uuid
);

impl RoleRepository {
    pub async fn list_all(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<Vec<Role>, crate::RepositoryError> {
        use crate::schema::roles::dsl::*;
        use diesel_async::RunQueryDsl;
        Ok(roles.load::<Role>(conn).await?)
    }
}
