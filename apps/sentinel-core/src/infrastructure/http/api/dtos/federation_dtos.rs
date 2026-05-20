//! DTOs for federation endpoints.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request body for exchanging a Supabase token.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct ExchangeSupabaseTokenRequest {
    /// Supabase JWT access token.
    #[validate(length(min = 1))]
    pub access_token: String,
}

/// Response from federation endpoints (same shape as login).
/// Re-exports BasicLoginResponse for consistency.
pub use crate::http::api::dtos::BasicLoginResponse as FederationLoginResponse;
