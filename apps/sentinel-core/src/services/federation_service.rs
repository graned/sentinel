//! Federation service for external identity providers.
//!
//! Handles the exchange of external tokens (e.g., Supabase JWT) for native Sentinel sessions.

use crate::{
    DbConnection, ExternalIdentity, ExternalIdentityRepository, IdentityProvider, IdentityService,
    RoleType, ServiceError, SessionService, Sessions, User, UserIdentity, UserRole,
    UserRoleService, UserService, UserStatus,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Service for federated authentication flows.
pub struct FederationService {
    external_identity_repo: Arc<ExternalIdentityRepository>,
    identity_service: Arc<IdentityService>,
    user_service: Arc<UserService>,
    user_role_service: Arc<UserRoleService>,
    pub session_service: Arc<SessionService>,
}

impl FederationService {
    pub fn new(
        external_identity_repo: Arc<ExternalIdentityRepository>,
        identity_service: Arc<IdentityService>,
        user_service: Arc<UserService>,
        user_role_service: Arc<UserRoleService>,
        session_service: Arc<SessionService>,
    ) -> Self {
        Self {
            external_identity_repo,
            identity_service,
            user_service,
            user_role_service,
            session_service,
        }
    }

    /// Exchange a verified external identity for a Sentinel session.
    ///
    /// Flow:
    /// 1. Check if external_identity exists (by provider+issuer+subject) → use that user
    /// 2. If not, check if user with that email exists → link external_identity to that user
    /// 3. If neither exists → create new user with that email
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `provider` - Provider name (e.g., "supabase")
    /// * `issuer` - Token issuer URL
    /// * `subject` - Stable external identity key (Supabase sub, stored for reference)
    /// * `email` - Email from token (used for user lookup/creation)
    /// * `user_metadata` - Additional metadata from token
    ///
    /// # Returns
    /// The created session and tokens.
    #[allow(clippy::too_many_arguments)]
    pub async fn exchange_external_identity(
        &self,
        conn: &mut DbConnection<'_>,
        provider: &str,
        issuer: &str,
        subject: &str,
        email: Option<String>,
        user_metadata: Option<serde_json::Value>,
    ) -> Result<(Sessions, crate::SessionTokens), ServiceError> {
        let existing_identity = self
            .external_identity_repo
            .find_by_provider_issuer_subject(conn, provider, issuer, subject)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if let Some(ext_identity) = existing_identity {
            self.exchange_existing_user(conn, ext_identity, email, user_metadata)
                .await
        } else {
            self.exchange_new_or_existing_user(conn, provider, issuer, subject, email, user_metadata)
                .await
        }
    }

    /// Handle exchange for an existing external identity.
    async fn exchange_existing_user(
        &self,
        conn: &mut DbConnection<'_>,
        ext_identity: ExternalIdentity,
        _email: Option<String>,
        user_metadata: Option<serde_json::Value>,
    ) -> Result<(Sessions, crate::SessionTokens), ServiceError> {
        let user = self
            .user_service
            .find_user_by_id(conn, ext_identity.user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                ServiceError::InternalError("User not found for external identity".to_string())
            })?;

        if user_metadata.is_some() {
            self.update_identity_metadata(conn, ext_identity.external_identity_id, user_metadata)
                .await?;
        }

        self.external_identity_repo
            .update_last_login(conn, ext_identity.external_identity_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let identity = self
            .identity_service
            .find_primary_identity_by_user_id(conn, user.user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::InternalError("Primary identity not found".to_string()))?;

        let roles = self
            .user_role_service
            .get_user_roles(conn, &user)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        self.create_session(conn, &user, &identity, &roles).await
    }

    /// Handle exchange for a new or existing user (by email).
    ///
    /// Flow:
    /// 1. Try to find user by email
    /// 2. If found, link external_identity to that user
    /// 3. If not found, create new user with that email
    #[allow(clippy::too_many_arguments)]
    async fn exchange_new_or_existing_user(
        &self,
        conn: &mut DbConnection<'_>,
        provider: &str,
        issuer: &str,
        subject: &str,
        email: Option<String>,
        user_metadata: Option<serde_json::Value>,
    ) -> Result<(Sessions, crate::SessionTokens), ServiceError> {
        let email_addr = email.clone().ok_or_else(|| {
            ServiceError::ValidationError("Email is required for federation".to_string())
        })?;

        let display_name = user_metadata.as_ref().and_then(|m| {
            m.get("full_name")
                .or_else(|| m.get("name"))
                .and_then(|v| v.as_str())
                .map(String::from)
        });

        let (persisted_user, identity) = match self.identity_service.find_identity_by_email(conn, &email_addr).await {
            Ok(Some(existing_identity)) => {
                let user = self
                    .user_service
                    .find_user_by_id(conn, existing_identity.user_id)
                    .await
                    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                    .ok_or_else(|| {
                        ServiceError::InternalError("User not found for identity".to_string())
                    })?;

                (user, existing_identity)
            }
            _ => {
                let user_id = Uuid::new_v4();

                let user = User {
                    user_id,
                    first_name: None,
                    last_name: None,
                    avatar_url: None,
                    status: UserStatus::Active,
                    token_version: 0,
                    mfa_required: false,
                    created_at: Some(Utc::now()),
                    updated_at: Some(Utc::now()),
                    created_by: Some(user_id),
                    updated_by: Some(user_id),
                    display_name,
                };

                let persisted_user = self
                    .user_service
                    .create_user(conn, &user)
                    .await
                    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

                let identity_id = Uuid::new_v4();
                let identity = UserIdentity {
                    identity_id,
                    user_id: persisted_user.user_id,
                    provider: IdentityProvider::EmailPassword,
                    provider_user_id: None,
                    email: email_addr.clone(),
                    password_hash: None,
                    password_changed_at: None,
                    email_verified: Some(true),
                    oauth_access_token: None,
                    oauth_refresh_token: None,
                    oauth_token_expires_at: None,
                    is_primary: true,
                    last_login_at: Some(Utc::now()),
                    must_change_password: false,
                    created_at: Some(Utc::now()),
                    updated_at: Some(Utc::now()),
                    created_by: Some(user_id),
                    updated_by: Some(user_id),
                };

                self.identity_service
                    .create_identity(conn, &identity)
                    .await
                    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

                (persisted_user, identity)
            }
        };

        let ext_identity = ExternalIdentity {
            external_identity_id: Uuid::new_v4(),
            user_id: persisted_user.user_id,
            provider: provider.to_string(),
            issuer: issuer.to_string(),
            subject: subject.to_string(),
            email_snapshot: email.clone(),
            metadata: user_metadata.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: Some(Utc::now()),
            created_by: Some(persisted_user.user_id),
            updated_by: Some(persisted_user.user_id),
        };

        self.external_identity_repo
            .create(conn, &ext_identity)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let roles = self
            .user_role_service
            .get_user_roles(conn, &persisted_user)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if roles.is_empty() {
            let role = self
                .user_role_service
                .get_role_by_type(conn, RoleType::User)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            let user_role = UserRole {
                user_role_id: Uuid::new_v4(),
                user_id: persisted_user.user_id,
                role_id: role.role_id,
                created_at: Utc::now(),
                created_by: Some(persisted_user.user_id),
            };

            self.user_role_service
                .add_role_to_user(conn, &user_role)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        let roles = self
            .user_role_service
            .get_user_roles(conn, &persisted_user)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        self.create_session(conn, &persisted_user, &identity, &roles).await
    }

    /// Update identity metadata snapshot.
    async fn update_identity_metadata(
        &self,
        conn: &mut DbConnection<'_>,
        external_identity_id: Uuid,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), ServiceError> {
        use crate::schema::external_identities::dsl::{
            external_identities, external_identity_id as col_id, metadata as col_metadata,
            updated_at,
        };
        use diesel::{ExpressionMethods, QueryDsl};
        use diesel_async::RunQueryDsl;

        diesel::update(external_identities.filter(col_id.eq(external_identity_id)))
            .set((col_metadata.eq(metadata), updated_at.eq(Utc::now())))
            .execute(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Create a session for the user.
    async fn create_session(
        &self,
        conn: &mut DbConnection<'_>,
        user: &User,
        identity: &UserIdentity,
        roles: &[crate::Role],
    ) -> Result<(Sessions, crate::SessionTokens), ServiceError> {
        let session_id = Uuid::new_v4();
        let tokens =
            self.session_service
                .generate_session_token(user, &session_id, roles, identity)?;

        let now = Utc::now();
        let session = Sessions {
            session_id,
            user_id: user.user_id,
            identity_id: identity.identity_id,
            refresh_token_hash: tokens.refresh_token_hash.clone(),
            refresh_token_family: Uuid::new_v4(),
            refresh_token_expires_at: now + self.session_service.refresh_ttl,
            user_agent: None,
            ip_address: None,
            device_type: None,
            revoked_at: None,
            revoked_reason: None,
            last_used_at: None,
            created_at: Some(now),
            updated_at: Some(now),
            created_by: Some(user.user_id),
            updated_by: Some(user.user_id),
        };

        self.session_service
            .create_session(conn, &session)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok((session, tokens))
    }
}
