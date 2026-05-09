//! User application — profile and session management for the authenticated user.
//!
//! All methods operate on the current user (identified via `AuthenticatedUserContext`)
//! and do not require admin privileges.
//!
//! # Methods
//!
//! - `get_profile` — fetch the user's email, name, and status from `user_identities`
//! - `update_me` — update the user's first_name, last_name, and avatar_url
//! - `get_sessions` — paginated list of all sessions (active and revoked)
//! - `get_session` — details of a specific session including device/IP metadata
//! - `get_permissions` — list the user's assigned roles
//! - `protected_canary` — demo endpoint: returns 200 if auth + authz pass

use crate::{
    http::api::dtos::{
        AuthenticatedUserContext, RoleResponse, UserPermissionsResponse, UserProfileResponse,
        UserSessionDetailResponse, UserSessionResponse,
    },
    IdentityService, MfaTotpService, PostgresClient, ServiceError, SessionService, UserRoleService,
    UserService,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct UserApplication {
    pg_client: Arc<PostgresClient>,
    user_service: Arc<UserService>,
    identity_service: Arc<IdentityService>,
    session_service: Arc<SessionService>,
    user_role_service: Arc<UserRoleService>,
    mfa_totp_service: Arc<MfaTotpService>,
}

impl UserApplication {
    pub fn new(
        pg_client: Arc<PostgresClient>,
        user_service: Arc<UserService>,
        identity_service: Arc<IdentityService>,
        session_service: Arc<SessionService>,
        user_role_service: Arc<UserRoleService>,
        mfa_totp_service: Arc<MfaTotpService>,
    ) -> Self {
        Self {
            pg_client,
            user_service,
            identity_service,
            session_service,
            user_role_service,
            mfa_totp_service,
        }
    }

    pub async fn get_me(
        &self,
        ctx: AuthenticatedUserContext,
    ) -> Result<UserProfileResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        let user = self
            .user_service
            .find_user_by_id(&mut conn, ctx.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User not found".to_string()))?;

        let identity = self
            .identity_service
            .find_primary_identity_by_user_id(&mut conn, ctx.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Identity not found".to_string()))?;

        let mfa_enabled = self
            .mfa_totp_service
            .is_mfa_enabled(&mut conn, ctx.user_id)
            .await?;

        Ok(UserProfileResponse {
            user_id: user.user_id,
            first_name: user.first_name,
            last_name: user.last_name,
            avatar_url: user.avatar_url,
            status: user.status,
            email: identity.email,
            email_verified: identity.email_verified.unwrap_or(false),
            mfa_enabled,
            created_at: user.created_at,
        })
    }

    pub async fn update_me(
        &self,
        ctx: AuthenticatedUserContext,
        first_name: Option<String>,
        last_name: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<UserProfileResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        let user = self
            .user_service
            .update_user_profile(
                &mut conn,
                ctx.user_id,
                first_name,
                last_name,
                avatar_url,
            )
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User not found".to_string()))?;

        let identity = self
            .identity_service
            .find_primary_identity_by_user_id(&mut conn, ctx.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Identity not found".to_string()))?;

        let mfa_enabled = self
            .mfa_totp_service
            .is_mfa_enabled(&mut conn, ctx.user_id)
            .await?;

        Ok(UserProfileResponse {
            user_id: user.user_id,
            first_name: user.first_name,
            last_name: user.last_name,
            avatar_url: user.avatar_url,
            status: user.status,
            email: identity.email,
            email_verified: identity.email_verified.unwrap_or(false),
            mfa_enabled,
            created_at: user.created_at,
        })
    }

    pub async fn get_sessions(
        &self,
        ctx: AuthenticatedUserContext,
    ) -> Result<Vec<UserSessionResponse>, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        let sessions = self
            .session_service
            .get_sessions_for_user(&mut conn, ctx.user_id)
            .await?;
        Ok(sessions
            .into_iter()
            .map(|s| UserSessionResponse {
                session_id: s.session_id,
                user_agent: s.user_agent,
                ip_address: s.ip_address,
                device_type: s.device_type,
                last_used_at: s.last_used_at,
                created_at: s.created_at,
                expires_at: s.refresh_token_expires_at,
                is_current: s.session_id == ctx.session_id,
            })
            .collect())
    }

    pub async fn get_session(
        &self,
        ctx: AuthenticatedUserContext,
        session_id: Uuid,
    ) -> Result<UserSessionDetailResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        let s = self
            .session_service
            .get_session_for_user(&mut conn, session_id, ctx.user_id)
            .await?;
        let is_active = s.revoked_at.is_none() && s.refresh_token_expires_at > Utc::now();
        Ok(UserSessionDetailResponse {
            session_id: s.session_id,
            user_agent: s.user_agent,
            ip_address: s.ip_address,
            device_type: s.device_type,
            last_used_at: s.last_used_at,
            created_at: s.created_at,
            expires_at: s.refresh_token_expires_at,
            revoked_at: s.revoked_at,
            is_current: s.session_id == ctx.session_id,
            is_active,
        })
    }

    pub async fn get_permissions(
        &self,
        ctx: AuthenticatedUserContext,
    ) -> Result<UserPermissionsResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        let roles = self
            .user_role_service
            .get_roles_by_user_id(&mut conn, ctx.user_id)
            .await?;
        Ok(UserPermissionsResponse {
            user_id: ctx.user_id,
            roles: roles
                .into_iter()
                .map(|r| RoleResponse {
                    role_id: r.role_id,
                    name: r.name,
                    role_type: r.type_.as_str().to_string(),
                    description: r.description,
                })
                .collect(),
        })
    }
}
