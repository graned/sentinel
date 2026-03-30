//! DTOs for API token endpoints.
//!
//! `CreateApiTokenResponse` includes the raw `token` field (returned exactly once).
//! `ApiTokenResponse` omits the token — it is never stored and cannot be retrieved.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Request body for creating a new API token.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateApiTokenRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response after creating an API token.
/// The `token` field contains the raw token — it is shown exactly once.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateApiTokenResponse {
    pub api_token_id: Uuid,
    /// Raw opaque token. Store it securely — this is the only time it is returned.
    pub token: String,
    pub name: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// List-item representation of an API token (no raw token).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ApiTokenResponse {
    pub api_token_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
