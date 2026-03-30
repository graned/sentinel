//! Admin application — user management, role assignment, and MFA admin controls.
//!
//! All public methods require the caller to hold the `admin` role (checked via the
//! policy engine in `authorize_middleware`; no in-application role check is needed).
//!
//! # Capabilities
//!
//! - **User management**: list (paginated), get, create, enable/disable, generate invite links
//! - **Role management**: create, list, update, delete roles; assign/remove from users
//! - **MFA admin controls**: disable a user's MFA (emergency), toggle `mfa_required` flag
//! - **Auth-info**: surface MFA status and identity details for support/audit workflows

use crate::{
    http::api::dtos::{
        AdminCreateUserRequest, AdminSetMfaRequiredRequest, AdminUserResponse, AssignRoleRequest,
        AuthenticatedUserContext, CreateRoleRequest, InviteLinkResponse, ListUsersQuery,
        PaginatedUsersResponse, RoleResponse, UpdateRoleRequest, UpdateUserStatusRequest,
        UserAuthInfoResponse, UserMfaStatusResponse, UserPermissionsResponse,
    },
    EmailService, EmailVerificationService, IdentityProvider, IdentityService, MfaTotpService,
    PostgresClient, Role, RoleType, ServiceError, SessionService, User, UserIdentity, UserRole,
    UserRoleService, UserService, UserStatus,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct AdminApplication {
    pg_client: Arc<PostgresClient>,
    user_service: Arc<UserService>,
    identity_service: Arc<IdentityService>,
    user_role_service: Arc<UserRoleService>,
    email_verification_service: Arc<EmailVerificationService>,
    email_service: Arc<EmailService>,
    session_service: Arc<SessionService>,
    mfa_totp_service: Arc<MfaTotpService>,
}

impl AdminApplication {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pg_client: Arc<PostgresClient>,
        user_service: Arc<UserService>,
        identity_service: Arc<IdentityService>,
        user_role_service: Arc<UserRoleService>,
        email_verification_service: Arc<EmailVerificationService>,
        email_service: Arc<EmailService>,
        session_service: Arc<SessionService>,
        mfa_totp_service: Arc<MfaTotpService>,
    ) -> Self {
        Self {
            pg_client,
            user_service,
            identity_service,
            user_role_service,
            email_verification_service,
            email_service,
            session_service,
            mfa_totp_service,
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

    fn parse_role_type(s: &str) -> RoleType {
        match s {
            "admin" => RoleType::Admin,
            "support" => RoleType::Support,
            _ => RoleType::User,
        }
    }

    fn role_to_response(role: Role) -> RoleResponse {
        RoleResponse {
            role_id: role.role_id,
            name: role.name,
            role_type: role.type_.as_str().to_string(),
            description: role.description,
        }
    }

    pub async fn create_role(
        &self,
        ctx: &AuthenticatedUserContext,
        req: CreateRoleRequest,
    ) -> Result<RoleResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let now = Utc::now();
        let role = Role {
            role_id: Uuid::new_v4(),
            type_: Self::parse_role_type(&req.role_type),
            name: req.name,
            description: req.description,
            created_at: now,
            updated_at: now,
            created_by: Some(ctx.user_id),
            updated_by: None,
        };
        let created = self.user_role_service.create_role(&mut conn, &role).await?;
        Ok(Self::role_to_response(created))
    }

    pub async fn list_roles(
        &self,
        ctx: &AuthenticatedUserContext,
    ) -> Result<Vec<RoleResponse>, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let roles = self.user_role_service.list_roles(&mut conn).await?;
        Ok(roles.into_iter().map(Self::role_to_response).collect())
    }

    pub async fn update_role(
        &self,
        ctx: &AuthenticatedUserContext,
        role_id: Uuid,
        req: UpdateRoleRequest,
    ) -> Result<RoleResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        self.user_role_service
            .find_role_by_id(&mut conn, role_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Role not found".to_string()))?;
        let updated = self
            .user_role_service
            .update_role(&mut conn, role_id, req.name, req.description)
            .await?;
        Ok(Self::role_to_response(updated))
    }

    pub async fn delete_role(
        &self,
        ctx: &AuthenticatedUserContext,
        role_id: Uuid,
    ) -> Result<(), ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        self.user_role_service
            .find_role_by_id(&mut conn, role_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Role not found".to_string()))?;
        self.user_role_service.delete_role(&mut conn, role_id).await
    }

    pub async fn assign_role_to_user(
        &self,
        ctx: &AuthenticatedUserContext,
        user_id: Uuid,
        req: AssignRoleRequest,
    ) -> Result<RoleResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        self.user_service
            .find_user_by_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User not found".to_string()))?;
        let role = self
            .user_role_service
            .find_role_by_id(&mut conn, req.role_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Role not found".to_string()))?;

        // Check for duplicate assignment
        if self
            .user_role_service
            .find_user_role(&mut conn, user_id, req.role_id)
            .await?
            .is_some()
        {
            return Err(ServiceError::InternalError(
                "Role already assigned to user".to_string(),
            ));
        }

        let now = Utc::now();
        let user_role = UserRole {
            user_role_id: Uuid::new_v4(),
            user_id,
            role_id: req.role_id,
            created_at: now,
            created_by: Some(ctx.user_id),
        };
        self.user_role_service
            .add_role_to_user(&mut conn, &user_role)
            .await?;
        Ok(Self::role_to_response(role))
    }

    pub async fn remove_role_from_user(
        &self,
        ctx: &AuthenticatedUserContext,
        user_id: Uuid,
        role_name: String,
    ) -> Result<(), ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        self.user_service
            .find_user_by_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User not found".to_string()))?;
        self.user_role_service
            .remove_role_from_user(&mut conn, user_id, &role_name)
            .await
    }

    pub async fn get_user_permissions(
        &self,
        ctx: &AuthenticatedUserContext,
        user_id: Uuid,
    ) -> Result<UserPermissionsResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        self.user_service
            .find_user_by_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User not found".to_string()))?;
        let roles = self
            .user_role_service
            .get_roles_by_user_id(&mut conn, user_id)
            .await?;
        Ok(UserPermissionsResponse {
            user_id,
            roles: roles.into_iter().map(Self::role_to_response).collect(),
        })
    }

    pub async fn get_user_auth_info(
        &self,
        ctx: &AuthenticatedUserContext,
        user_id: Uuid,
    ) -> Result<UserAuthInfoResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let user = self
            .user_service
            .find_user_by_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User not found".to_string()))?;
        let identity = self
            .identity_service
            .find_primary_identity_by_user_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Identity not found".to_string()))?;
        let roles = self
            .user_role_service
            .get_roles_by_user_id(&mut conn, user_id)
            .await?;
        Ok(UserAuthInfoResponse {
            user_id: user.user_id,
            first_name: user.first_name,
            last_name: user.last_name,
            avatar_url: user.avatar_url,
            status: user.status,
            email: identity.email,
            email_verified: identity.email_verified.unwrap_or(false),
            created_at: user.created_at,
            roles: roles.into_iter().map(Self::role_to_response).collect(),
        })
    }

    // ── User management ──────────────────────────────────────────────────────

    fn user_to_admin_response(
        user: User,
        email: String,
        email_verified: bool,
        roles: Vec<Role>,
        mfa_enabled: bool,
        mfa_required: bool,
    ) -> AdminUserResponse {
        AdminUserResponse {
            user_id: user.user_id,
            first_name: user.first_name,
            last_name: user.last_name,
            email,
            email_verified,
            status: user.status,
            roles: roles.into_iter().map(Self::role_to_response).collect(),
            mfa_enabled,
            mfa_required,
            created_at: user.created_at,
        }
    }

    pub async fn list_users(
        &self,
        ctx: &AuthenticatedUserContext,
        query: ListUsersQuery,
    ) -> Result<PaginatedUsersResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

        let mut conn = self.pg_client.get_conn().await?;
        let (users, total) = self
            .user_service
            .paginate_users(&mut conn, page, page_size)
            .await?;

        let mut items = Vec::with_capacity(users.len());
        for user in users {
            let identity = self
                .identity_service
                .find_primary_identity_by_user_id(&mut conn, user.user_id)
                .await?;
            let roles = self
                .user_role_service
                .get_roles_by_user_id(&mut conn, user.user_id)
                .await?;
            let mfa_enabled = self
                .mfa_totp_service
                .is_mfa_enabled(&mut conn, user.user_id)
                .await?;
            let (email, email_verified) = identity
                .map(|i| (i.email, i.email_verified.unwrap_or(false)))
                .unwrap_or_else(|| (String::new(), false));
            let mfa_required = user.mfa_required;
            items.push(Self::user_to_admin_response(
                user,
                email,
                email_verified,
                roles,
                mfa_enabled,
                mfa_required,
            ));
        }
        Ok(PaginatedUsersResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_user(
        &self,
        ctx: &AuthenticatedUserContext,
        req: AdminCreateUserRequest,
    ) -> Result<AdminUserResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;

        // Check email availability
        self.identity_service
            .verify_email_availability(&mut conn, &req.email)
            .await?;

        let now = Utc::now();
        let user_id = Uuid::new_v4();

        let new_user = User {
            user_id,
            first_name: Some(req.first_name.clone()),
            last_name: Some(req.last_name.clone()),
            avatar_url: None,
            // Admin-invited users start as PendingVerification until email is confirmed
            status: UserStatus::PendingVerification,
            token_version: 0,
            mfa_required: false,
            created_by: Some(ctx.user_id),
            created_at: Some(now),
            updated_by: Some(ctx.user_id),
            updated_at: Some(now),
        };
        let persisted_user = self.user_service.create_user(&mut conn, &new_user).await?;

        let identity_id = Uuid::new_v4();
        let new_identity = UserIdentity {
            identity_id,
            user_id,
            provider: IdentityProvider::EmailPassword,
            provider_user_id: None,
            email: req.email.clone(),
            // Raw password — the DB trigger hashes it via pgcrypt
            password_hash: Some(req.password.clone()),
            password_changed_at: None,
            // Email is not pre-verified; must be confirmed via invite link
            email_verified: Some(false),
            oauth_access_token: None,
            oauth_refresh_token: None,
            oauth_token_expires_at: None,
            is_primary: true,
            // Admin-set temp password must be changed on first login
            must_change_password: true,
            last_login_at: None,
            created_by: Some(ctx.user_id),
            created_at: Some(now),
            updated_by: Some(ctx.user_id),
            updated_at: Some(now),
        };
        self.identity_service
            .create_identity(&mut conn, &new_identity)
            .await?;

        // Assign the default "user" role
        let role = self
            .user_role_service
            .get_role_by_type(&mut conn, RoleType::User)
            .await?;
        let user_role = UserRole {
            user_role_id: Uuid::new_v4(),
            user_id,
            role_id: role.role_id,
            created_at: now,
            created_by: Some(ctx.user_id),
        };
        self.user_role_service
            .add_role_to_user(&mut conn, &user_role)
            .await?;

        let roles = self
            .user_role_service
            .get_roles_by_user_id(&mut conn, user_id)
            .await?;

        // Optionally send the invite/verification email
        if req.send_invite_email.unwrap_or(false) {
            let first_name = persisted_user.first_name.clone().unwrap_or_default();
            match self
                .email_verification_service
                .create_verification(&mut conn, identity_id, user_id)
                .await
            {
                Ok(raw_token) => {
                    let _ = self
                        .email_service
                        .send_verification_email(&mut conn, &req.email, &first_name, &raw_token)
                        .await
                        .map_err(|e| {
                            tracing::warn!("Invite email send failed for {}: {e}", req.email);
                        });
                }
                Err(e) => {
                    tracing::warn!("Failed to create invite verification token: {e}");
                }
            }
        }

        Ok(Self::user_to_admin_response(
            persisted_user,
            req.email,
            false,
            roles,
            false,
            false,
        ))
    }

    pub async fn delete_user(
        &self,
        ctx: &AuthenticatedUserContext,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        self.user_service
            .find_user_by_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User not found".to_string()))?;
        self.user_service.delete_user(&mut conn, user_id).await
    }

    pub async fn update_user_status(
        &self,
        ctx: &AuthenticatedUserContext,
        user_id: Uuid,
        req: UpdateUserStatusRequest,
    ) -> Result<AdminUserResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let new_status = match req.status.as_str() {
            "active" => UserStatus::Active,
            "suspended" => UserStatus::Suspended,
            "inactive" => UserStatus::Inactive,
            other => {
                return Err(ServiceError::ValidationError(format!(
                    "Invalid status '{}'. Use: active, suspended, inactive",
                    other
                )))
            }
        };
        let mut conn = self.pg_client.get_conn().await?;
        let updated_user = self
            .user_service
            .update_user_status(&mut conn, user_id, new_status)
            .await?;
        let identity = self
            .identity_service
            .find_primary_identity_by_user_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Identity not found".to_string()))?;
        let roles = self
            .user_role_service
            .get_roles_by_user_id(&mut conn, user_id)
            .await?;
        let mfa_enabled = self
            .mfa_totp_service
            .is_mfa_enabled(&mut conn, user_id)
            .await?;
        let mfa_required = updated_user.mfa_required;
        Ok(Self::user_to_admin_response(
            updated_user,
            identity.email,
            identity.email_verified.unwrap_or(false),
            roles,
            mfa_enabled,
            mfa_required,
        ))
    }

    /// POST /v1/api/admin/users/{user_id}/send-invite
    /// Creates a fresh verification token and emails it to the user.
    pub async fn send_invite(
        &self,
        ctx: &AuthenticatedUserContext,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let identity = self
            .identity_service
            .find_primary_identity_by_user_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User identity not found".to_string()))?;

        if identity.email_verified.unwrap_or(false) {
            return Err(ServiceError::ValidationError(
                "Email is already verified".to_string(),
            ));
        }

        let user = self
            .user_service
            .find_user_by_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User not found".to_string()))?;
        let first_name = user.first_name.unwrap_or_default();

        let raw_token = self
            .email_verification_service
            .create_verification(&mut conn, identity.identity_id, user_id)
            .await?;

        self.email_service
            .send_verification_email(&mut conn, &identity.email, &first_name, &raw_token)
            .await
    }

    pub async fn set_mfa_required(
        &self,
        ctx: &AuthenticatedUserContext,
        user_id: Uuid,
        req: AdminSetMfaRequiredRequest,
    ) -> Result<UserMfaStatusResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        self.user_service
            .find_user_by_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User not found".to_string()))?;
        self.user_service
            .set_mfa_required(&mut conn, user_id, req.required)
            .await?;
        if req.required {
            self.session_service
                .revoke_all_sessions(&mut conn, user_id)
                .await?;
        }
        let mfa_enabled = self
            .mfa_totp_service
            .is_mfa_enabled(&mut conn, user_id)
            .await?;
        Ok(UserMfaStatusResponse {
            mfa_required: req.required,
            mfa_enabled,
        })
    }

    /// GET /v1/api/admin/users/{user_id}/invite-link
    /// Generates a verification token and returns the raw invite URL (no email sent).
    pub async fn get_invite_link(
        &self,
        ctx: &AuthenticatedUserContext,
        user_id: Uuid,
    ) -> Result<InviteLinkResponse, ServiceError> {
        Self::require_admin(ctx)?;
        let mut conn = self.pg_client.get_conn().await?;
        let identity = self
            .identity_service
            .find_primary_identity_by_user_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("User identity not found".to_string()))?;

        if identity.email_verified.unwrap_or(false) {
            return Err(ServiceError::ValidationError(
                "Email is already verified".to_string(),
            ));
        }

        let raw_token = self
            .email_verification_service
            .create_verification(&mut conn, identity.identity_id, user_id)
            .await?;

        let invite_url = format!(
            "{}/verify-email?token={}",
            self.email_service.frontend_url(),
            raw_token
        );
        Ok(InviteLinkResponse { invite_url })
    }
}
