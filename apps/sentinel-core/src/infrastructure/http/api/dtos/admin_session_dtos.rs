//! DTOs for admin session endpoints: listing all active sessions, force-revoking
//! individual sessions, and bulk revocation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AdminSessionResponse {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct BulkRevokeSessionsRequest {
    #[validate(length(min = 1))]
    pub session_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BulkRevokeSessionsResponse {
    pub revoked_count: usize,
}
