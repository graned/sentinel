//! Repository for the `sessions` table.
//!
//! Sessions are **soft-deleted only** — revocation sets `revoked_at` and
//! `revoked_reason`; rows are never hard-deleted. This preserves audit trails
//! and allows detecting replay attacks on revoked refresh tokens.
//!
//! The [`impl_repository!`] macro provides standard CRUD. Custom methods add:
//! - Aggregate insights (active session count, daily activity, distinct active users)
//! - Admin views (all active sessions with user email via JOIN)
//! - Bulk revocation helpers used by logout-all and password-change flows

use crate::impl_repository;
use crate::{RevocationReason, Sessions};

use uuid::Uuid;

/// Aggregate row returned by [`SessionRepository::daily_session_activity`].
#[derive(Debug, diesel::QueryableByName)]
pub struct DailySessionActivity {
    #[diesel(sql_type = diesel::sql_types::Date)]
    pub date: chrono::NaiveDate,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub sessions_created: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub unique_users: i64,
}

impl_repository!(
    SessionRepository for Sessions,
    crate::schema::sessions::table,
    crate::schema::sessions::session_id,
    Uuid
);

impl SessionRepository {
    /// Count non-revoked, non-expired (live) sessions.
    pub async fn count_active(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<i64, diesel::result::Error> {
        use crate::schema::sessions::dsl::{refresh_token_expires_at, revoked_at, sessions};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        sessions
            .filter(revoked_at.is_null())
            .filter(refresh_token_expires_at.gt(chrono::Utc::now()))
            .count()
            .get_result(conn)
            .await
    }

    /// Count distinct users who had any session activity (`last_used_at`) on or after `since`.
    pub async fn count_distinct_active_users_since(
        &self,
        conn: &mut crate::DbConnection<'_>,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<i64, diesel::result::Error> {
        use crate::schema::sessions::dsl::{last_used_at, sessions, user_id};
        use diesel::dsl::count_distinct;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        sessions
            .filter(last_used_at.ge(since))
            .select(count_distinct(user_id))
            .get_result(conn)
            .await
    }

    /// Return a per-day count of sessions created and distinct users over the last `days` days.
    /// Uses raw SQL for `DATE()` truncation + `GROUP BY`.
    pub async fn daily_session_activity(
        &self,
        conn: &mut crate::DbConnection<'_>,
        days: i32,
    ) -> Result<Vec<DailySessionActivity>, diesel::result::Error> {
        use diesel_async::RunQueryDsl;

        diesel::sql_query(
            r#"
            SELECT DATE(created_at)         AS date,
                   COUNT(*)                 AS sessions_created,
                   COUNT(DISTINCT user_id)  AS unique_users
            FROM   sessions
            WHERE  created_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP  BY DATE(created_at)
            ORDER  BY DATE(created_at)
            "#,
        )
        .bind::<diesel::sql_types::Integer, _>(days)
        .load(conn)
        .await
    }

    /// Returns all non-revoked, non-expired sessions joined with user_identities
    /// to include the user's email address. Used by the admin sessions view.
    pub async fn find_all_active(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<Vec<(Sessions, String)>, diesel::result::Error> {
        use crate::schema::{sessions, user_identities};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        sessions::table
            .filter(sessions::revoked_at.is_null())
            .filter(sessions::refresh_token_expires_at.gt(chrono::Utc::now()))
            .inner_join(
                user_identities::table
                    .on(user_identities::identity_id.eq(sessions::identity_id)),
            )
            .select((Sessions::as_select(), user_identities::email))
            .order(sessions::created_at.desc())
            .load(conn)
            .await
    }

    /// Bulk-revokes all sessions in the provided ID list.
    /// Uses `.execute()` (not `.get_result()`) so it works for multiple rows.
    pub async fn revoke_sessions_by_ids(
        &self,
        conn: &mut crate::DbConnection<'_>,
        ids: &[Uuid],
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::sessions::dsl::{
            revoked_at as col_revoked_at, revoked_reason as col_revoked_reason,
            session_id as col_session_id, sessions,
        };
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        diesel::update(sessions.filter(col_session_id.eq_any(ids)))
            .set((
                col_revoked_at.eq(chrono::Utc::now()),
                col_revoked_reason.eq(RevocationReason::UserLogout),
            ))
            .execute(conn)
            .await
    }

    /// Bulk-revoke all non-revoked sessions for a user.
    /// Called by logout-all, password-change, and password-reset flows to immediately
    /// invalidate every live session across all devices.
    pub async fn revoke_all_active_sessions_for_user(
        &self,
        conn: &mut crate::DbConnection<'_>,
        target_user_id: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::sessions::dsl::{
            revoked_at as col_revoked_at, revoked_reason as col_revoked_reason, sessions, user_id,
        };
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        diesel::update(
            sessions
                .filter(user_id.eq(target_user_id))
                .filter(col_revoked_at.is_null()),
        )
        .set((
            col_revoked_at.eq(chrono::Utc::now()),
            col_revoked_reason.eq(RevocationReason::UserLogout),
        ))
        .execute(conn)
        .await
    }
}
