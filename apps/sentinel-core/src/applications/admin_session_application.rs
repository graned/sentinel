//! Admin session application — cross-user session visibility and force-revocation.
//!
//! Allows admins to view all active sessions (with user email) and force-revoke
//! individual or bulk sessions.  This is used for security incident response and
//! account compromise scenarios.
//!
//! Sessions are soft-deleted only — `revoked_at` is set; rows are never hard-deleted.

use crate::{
    http::api::dtos::{
        AdminSessionResponse, AuthenticatedUserContext, BulkRevokeSessionsRequest,
        BulkRevokeSessionsResponse,
    },
    PostgresClient, ServiceError, SessionService,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct AdminSessionApplication {
    pg_client: Arc<PostgresClient>,
    session_service: Arc<SessionService>,
}

impl AdminSessionApplication {
    pub fn new(pg_client: Arc<PostgresClient>, session_service: Arc<SessionService>) -> Self {
        Self {
            pg_client,
            session_service,
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

    pub async fn get_all_active_sessions(
        &self,
        ctx: AuthenticatedUserContext,
    ) -> Result<Vec<AdminSessionResponse>, ServiceError> {
        Self::require_admin(&ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let rows = self
            .session_service
            .get_all_active_sessions(&mut conn)
            .await?;
        Ok(rows
            .into_iter()
            .map(|(s, email)| AdminSessionResponse {
                session_id: s.session_id,
                user_id: s.user_id,
                user_email: email,
                user_agent: s.user_agent,
                ip_address: s.ip_address,
                device_type: s.device_type,
                last_used_at: s.last_used_at,
                created_at: s.created_at,
                expires_at: s.refresh_token_expires_at,
            })
            .collect())
    }

    pub async fn revoke_session(
        &self,
        ctx: AuthenticatedUserContext,
        session_id: Uuid,
    ) -> Result<(), ServiceError> {
        Self::require_admin(&ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        self.session_service
            .admin_revoke_session(&mut conn, session_id)
            .await
    }

    pub async fn revoke_sessions_bulk(
        &self,
        ctx: AuthenticatedUserContext,
        req: BulkRevokeSessionsRequest,
    ) -> Result<BulkRevokeSessionsResponse, ServiceError> {
        Self::require_admin(&ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let revoked_count = self
            .session_service
            .admin_revoke_sessions_bulk(&mut conn, &req.session_ids)
            .await?;
        Ok(BulkRevokeSessionsResponse { revoked_count })
    }
}
