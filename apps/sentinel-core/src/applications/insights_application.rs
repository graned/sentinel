//! Insights application — dashboard aggregate metrics.
//!
//! Aggregates statistics from multiple repositories into a single summary response
//! for the admin dashboard.  All queries run against the live database — there is no
//! caching layer.
//!
//! # Metrics returned
//!
//! | Metric | Source |
//! |--------|--------|
//! | `total_users` | `users` table count |
//! | `active_sessions` | non-revoked, non-expired `sessions` rows |
//! | `mfa_enabled_users` | `user_mfa_totp` rows with `enabled = true` |
//! | `verified_emails` | `user_identities` rows with `email_verified = true` |
//! | `new_users_30d` | `users.created_at >= now() - 30 days` |
//! | `active_users_7d` | distinct `user_id` in `sessions.last_used_at >= now() - 7 days` |
//! | `user_growth` | per-day new-user counts for the last N days |
//! | `session_activity` | per-day session count + unique users for the last N days |

use crate::{
    domain::repositories::{
        identities_repository::IdentitiesRepository, session_repository::SessionRepository,
        user_mfa_totp_repository::UserMfaTotpRepository, user_repository::UserRepository,
    },
    http::api::dtos::{
        AuthenticatedUserContext, InsightsSummaryResponse, SessionActivityPoint, UserGrowthPoint,
    },
    PostgresClient, ServiceError,
};
use chrono::Utc;
use std::sync::Arc;

pub struct InsightsApplication {
    pg_client: Arc<PostgresClient>,
    user_repo: Arc<UserRepository>,
    session_repo: Arc<SessionRepository>,
    mfa_totp_repo: Arc<UserMfaTotpRepository>,
    identities_repo: Arc<IdentitiesRepository>,
}

impl InsightsApplication {
    pub fn new(
        pg_client: Arc<PostgresClient>,
        user_repo: Arc<UserRepository>,
        session_repo: Arc<SessionRepository>,
        mfa_totp_repo: Arc<UserMfaTotpRepository>,
        identities_repo: Arc<IdentitiesRepository>,
    ) -> Self {
        Self {
            pg_client,
            user_repo,
            session_repo,
            mfa_totp_repo,
            identities_repo,
        }
    }

    fn require_admin(ctx: &AuthenticatedUserContext) -> Result<(), ServiceError> {
        if !ctx.roles.iter().any(|r| r == "admin") {
            return Err(ServiceError::AuthorizationError(
                "Admin role required".into(),
            ));
        }
        Ok(())
    }

    /// Return platform-wide KPI snapshot.
    ///
    /// `GET /v1/api/system/stats`
    pub async fn get_summary(
        &self,
        ctx: &AuthenticatedUserContext,
    ) -> Result<InsightsSummaryResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;

        let now = Utc::now();
        let week_ago = now - chrono::Duration::days(7);
        let month_ago = now - chrono::Duration::days(30);

        let total_users = self
            .user_repo
            .count(&mut conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let new_users_week = self
            .user_repo
            .count_created_since(&mut conn, week_ago)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let new_users_month = self
            .user_repo
            .count_created_since(&mut conn, month_ago)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let active_users_week = self
            .session_repo
            .count_distinct_active_users_since(&mut conn, week_ago)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let active_users_month = self
            .session_repo
            .count_distinct_active_users_since(&mut conn, month_ago)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let active_sessions = self
            .session_repo
            .count_active(&mut conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let mfa_enabled = self
            .mfa_totp_repo
            .count_enabled(&mut conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let mfa_adoption_pct = if total_users > 0 {
            (mfa_enabled as f64 / total_users as f64) * 100.0
        } else {
            0.0
        };

        let total_identities = self
            .identities_repo
            .count(&mut conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let verified_identities = self
            .identities_repo
            .count_email_verified(&mut conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let email_verified_pct = if total_identities > 0 {
            (verified_identities as f64 / total_identities as f64) * 100.0
        } else {
            0.0
        };

        Ok(InsightsSummaryResponse {
            total_users,
            new_users_week,
            new_users_month,
            active_users_week,
            active_users_month,
            active_sessions,
            mfa_adoption_pct,
            email_verified_pct,
        })
    }

    /// Return a cumulative user-growth time series.
    ///
    /// `GET /v1/api/system/analytics/user-growth?days=N`
    pub async fn get_user_growth(
        &self,
        ctx: &AuthenticatedUserContext,
        days: i32,
    ) -> Result<Vec<UserGrowthPoint>, ServiceError> {
        Self::require_admin(ctx)?;
        let days = days.clamp(1, 365);
        let mut conn = self.pg_client.get_conn().await?;

        // Total users *before* the window starts (baseline for cumulative sum).
        let window_start = Utc::now() - chrono::Duration::days(days as i64);
        let baseline = self
            .user_repo
            .count_created_since(&mut conn, chrono::DateTime::UNIX_EPOCH) // all time
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            - self
                .user_repo
                .count_created_since(&mut conn, window_start)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let daily = self
            .user_repo
            .daily_new_users(&mut conn, days)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Build cumulative sum: each day's total = baseline + sum of new users up to that day.
        let mut cumulative = baseline;
        let points = daily
            .into_iter()
            .map(|row| {
                cumulative += row.count;
                UserGrowthPoint {
                    date: row.date.to_string(),
                    total_users: cumulative,
                    new_users: row.count,
                }
            })
            .collect();

        Ok(points)
    }

    /// Return a daily session-activity time series.
    ///
    /// `GET /v1/api/system/analytics/sessions?days=N`
    pub async fn get_session_activity(
        &self,
        ctx: &AuthenticatedUserContext,
        days: i32,
    ) -> Result<Vec<SessionActivityPoint>, ServiceError> {
        Self::require_admin(ctx)?;
        let days = days.clamp(1, 365);
        let mut conn = self.pg_client.get_conn().await?;

        let rows = self
            .session_repo
            .daily_session_activity(&mut conn, days)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let points = rows
            .into_iter()
            .map(|row| SessionActivityPoint {
                date: row.date.to_string(),
                sessions_created: row.sessions_created,
                unique_users: row.unique_users,
            })
            .collect();

        Ok(points)
    }
}
