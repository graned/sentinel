//! Repository for the `users` table.
//!
//! The [`impl_repository!`] macro provides standard CRUD + pagination.
//! Custom methods cover admin-specific queries (status/MFA-required updates,
//! full user listing, and aggregate insights for the dashboard).

use crate::impl_repository;
use crate::{RepositoryError, User, UserStatus};

use uuid::Uuid;

/// Aggregate row returned by [`UserRepository::daily_new_users`].
#[derive(Debug, diesel::QueryableByName)]
pub struct DailyNewUsers {
    #[diesel(sql_type = diesel::sql_types::Date)]
    pub date: chrono::NaiveDate,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

impl_repository!(
    UserRepository for User,
    crate::schema::users::table,
    crate::schema::users::user_id,
    Uuid
);

impl UserRepository {
    /// Count users whose `created_at` is on or after `since`.
    ///
    /// `created_at` is nullable in the schema; NULL rows are excluded by the
    /// `>= since` comparison (NULL comparisons evaluate to NULL / false).
    pub async fn count_created_since(
        &self,
        conn: &mut crate::DbConnection<'_>,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<i64, diesel::result::Error> {
        use crate::schema::users::dsl::{created_at, users};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        users
            .filter(created_at.ge(since))
            .count()
            .get_result(conn)
            .await
    }

    /// Return a per-day count of new registrations over the last `days` days.
    /// Uses raw SQL for the `DATE()` truncation + `GROUP BY`.
    pub async fn daily_new_users(
        &self,
        conn: &mut crate::DbConnection<'_>,
        days: i32,
    ) -> Result<Vec<DailyNewUsers>, diesel::result::Error> {
        use diesel_async::RunQueryDsl;

        diesel::sql_query(
            r#"
            SELECT DATE(created_at) AS date,
                   COUNT(*)         AS count
            FROM   users
            WHERE  created_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP  BY DATE(created_at)
            ORDER  BY DATE(created_at)
            "#,
        )
        .bind::<diesel::sql_types::Integer, _>(days)
        .load(conn)
        .await
    }

    /// Return every user row — used by the admin user-list view.
    /// Prefer `paginate_all` for large datasets.
    pub async fn list_all<'a>(
        &self,
        conn: &mut crate::DbConnection<'a>,
    ) -> Result<Vec<User>, RepositoryError> {
        use diesel_async::RunQueryDsl;
        crate::schema::users::table
            .load::<User>(conn)
            .await
            .map_err(RepositoryError::from)
    }

    /// Set `users.status` for a single user — used by admin enable/disable actions.
    pub async fn update_status<'a>(
        &self,
        conn: &mut crate::DbConnection<'a>,
        target_user_id: Uuid,
        new_status: UserStatus,
    ) -> Result<User, RepositoryError> {
        use crate::schema::users::dsl::{status, user_id, users};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        diesel::update(users)
            .filter(user_id.eq(target_user_id))
            .set(status.eq(new_status))
            .get_result::<User>(conn)
            .await
            .map_err(RepositoryError::from)
    }

    /// Set `users.mfa_required` — used by admin to mandate/release MFA enrollment.
    pub async fn update_mfa_required<'a>(
        &self,
        conn: &mut crate::DbConnection<'a>,
        target_user_id: Uuid,
        required: bool,
    ) -> Result<User, RepositoryError> {
        use crate::schema::users::dsl::{mfa_required, user_id, users};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        diesel::update(users)
            .filter(user_id.eq(target_user_id))
            .set(mfa_required.eq(required))
            .get_result::<User>(conn)
            .await
            .map_err(RepositoryError::from)
    }
}
