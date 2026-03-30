//! DTOs for the insights/analytics endpoints: KPI summary, user-growth time
//! series, and session-activity time series.

use serde::{Deserialize, Serialize};

// ─── Response DTOs ────────────────────────────────────────────────────────────

/// Platform-level KPI snapshot. All counts reflect live DB state.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct InsightsSummaryResponse {
    /// Total registered users (all statuses).
    pub total_users: i64,
    /// Users who registered in the last 7 days.
    pub new_users_week: i64,
    /// Users who registered in the last 30 days.
    pub new_users_month: i64,
    /// Distinct users with a session active (last_used_at) in the last 7 days.
    pub active_users_week: i64,
    /// Distinct users with a session active (last_used_at) in the last 30 days.
    pub active_users_month: i64,
    /// Live sessions: not revoked and refresh token not expired.
    pub active_sessions: i64,
    /// Percentage of users with TOTP MFA enabled (0–100).
    pub mfa_adoption_pct: f64,
    /// Percentage of user identities with a verified email (0–100).
    pub email_verified_pct: f64,
}

/// One day's slice of the cumulative user-growth time series.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserGrowthPoint {
    /// Calendar date, `YYYY-MM-DD`.
    pub date: String,
    /// Cumulative total users as of end-of-day.
    pub total_users: i64,
    /// New registrations on this specific day.
    pub new_users: i64,
}

/// One day's slice of the session-activity time series.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SessionActivityPoint {
    /// Calendar date, `YYYY-MM-DD`.
    pub date: String,
    /// Number of new sessions created on this day.
    pub sessions_created: i64,
    /// Distinct users who created at least one session on this day.
    pub unique_users: i64,
}

// ─── Query params ─────────────────────────────────────────────────────────────

/// Query parameters for time-series analytics endpoints.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct InsightsParams {
    /// Number of days of history to return (default: 30, max: 365).
    #[serde(default = "default_days")]
    pub days: i32,
}

fn default_days() -> i32 {
    30
}
