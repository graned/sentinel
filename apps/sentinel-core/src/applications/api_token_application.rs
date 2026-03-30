//! API token application — use-case orchestration for long-lived token management.
//!
//! All methods gate on the `admin` role via `require_admin`.  The authorization check
//! lives here (not in middleware) because the logic is role-specific rather than
//! path-specific.
//!
//! # Token lifecycle
//!
//! 1. `create` — generate `sat_<hex>`, store the SHA-256 hash, return the raw value once.
//! 2. `list` — return all tokens for the caller (including revoked); raw values are never returned.
//! 3. `revoke` — soft-delete one token (`revoked_at = now()`).
//! 4. `revoke_all` — soft-delete all tokens for the caller.

use crate::{
    http::api::dtos::{
        ApiTokenResponse, AuthenticatedUserContext, CreateApiTokenRequest, CreateApiTokenResponse,
    },
    ApiToken, ApiTokenService, PostgresClient, ServiceError,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ApiTokenApplication {
    pg_client: Arc<PostgresClient>,
    api_token_service: Arc<ApiTokenService>,
}

impl ApiTokenApplication {
    pub fn new(pg_client: Arc<PostgresClient>, api_token_service: Arc<ApiTokenService>) -> Self {
        Self {
            pg_client,
            api_token_service,
        }
    }

    fn require_admin(ctx: &AuthenticatedUserContext) -> Result<(), ServiceError> {
        if !ctx.roles.iter().any(|r| r == "admin") {
            return Err(ServiceError::AuthorizationError(
                "Admin role required".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn create_api_token(
        &self,
        ctx: AuthenticatedUserContext,
        req: CreateApiTokenRequest,
    ) -> Result<CreateApiTokenResponse, ServiceError> {
        Self::require_admin(&ctx)?;

        let (raw_token, token_hash) = self.api_token_service.generate_token();

        let now = Utc::now();
        let new_token = ApiToken {
            api_token_id: Uuid::new_v4(),
            user_id: ctx.user_id,
            name: req.name,
            description: req.description,
            token_hash,
            expires_at: req.expires_at,
            last_used_at: None,
            revoked_at: None,
            created_at: now,
            updated_at: None,
            created_by: Some(ctx.user_id),
            updated_by: None,
        };

        let mut conn = self.pg_client.get_conn().await?;
        let created = self
            .api_token_service
            .create(&mut conn, &new_token)
            .await?;

        Ok(CreateApiTokenResponse {
            api_token_id: created.api_token_id,
            token: raw_token,
            name: created.name,
            expires_at: created.expires_at,
            created_at: created.created_at,
        })
    }

    pub async fn list_api_tokens(
        &self,
        ctx: AuthenticatedUserContext,
    ) -> Result<Vec<ApiTokenResponse>, ServiceError> {
        Self::require_admin(&ctx)?;

        let mut conn = self.pg_client.get_conn().await?;
        let tokens = self
            .api_token_service
            .list_for_user(&mut conn, ctx.user_id)
            .await?;

        Ok(tokens
            .into_iter()
            .map(|t| ApiTokenResponse {
                api_token_id: t.api_token_id,
                name: t.name,
                description: t.description,
                expires_at: t.expires_at,
                last_used_at: t.last_used_at,
                revoked_at: t.revoked_at,
                created_at: t.created_at,
            })
            .collect())
    }

    pub async fn revoke_api_token(
        &self,
        ctx: AuthenticatedUserContext,
        token_id: Uuid,
    ) -> Result<(), ServiceError> {
        Self::require_admin(&ctx)?;

        let mut conn = self.pg_client.get_conn().await?;
        self.api_token_service
            .revoke(&mut conn, token_id, ctx.user_id)
            .await?;
        Ok(())
    }

    pub async fn revoke_all_api_tokens(
        &self,
        ctx: AuthenticatedUserContext,
    ) -> Result<(), ServiceError> {
        Self::require_admin(&ctx)?;

        let mut conn = self.pg_client.get_conn().await?;
        self.api_token_service
            .revoke_all_for_user(&mut conn, ctx.user_id)
            .await?;
        Ok(())
    }
}
