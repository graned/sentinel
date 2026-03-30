//! Repository for the `user_identities` table.
//!
//! A user identity holds the email address and password hash for local authentication.
//! The custom `fetch_identity_with_credentials` method is the authentication hot-path —
//! it passes the plaintext password to PostgreSQL's `crypt()` function inside the SQL
//! query so the hash comparison never leaves the database.

use crate::impl_repository;
use crate::UserIdentity;
use diesel::sql_types::Text;
use diesel::OptionalExtension;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

impl_repository!(
    IdentitiesRepository for UserIdentity,
    crate::schema::user_identities::table,
    crate::schema::user_identities::identity_id,
    Uuid
);

impl IdentitiesRepository {
    /// Count user identities where `email_verified = true`.
    pub async fn count_email_verified(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<i64, diesel::result::Error> {
        use crate::schema::user_identities::dsl::{email_verified, user_identities};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        user_identities
            .filter(email_verified.eq(true))
            .count()
            .get_result(conn)
            .await
    }

    pub async fn fetch_identity_with_credentials(
        &self,
        conn: &mut DbConnection<'_>,
        email: &str,
        password: &str,
    ) -> Result<Option<UserIdentity>, diesel::result::Error> {
        let q = diesel::sql_query(
            r#"
            SELECT *
            FROM user_identities
            WHERE email = $1
              AND password_hash = crypt($2, password_hash)
            LIMIT 1
            "#,
        )
        .bind::<Text, _>(email)
        .bind::<Text, _>(password);

        // `optional()` turns NotFound into Ok(None)
        q.get_result::<UserIdentity>(conn).await.optional()
    }
}
