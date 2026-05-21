//! Authentication application layer — orchestrates multi-service flows for
//! registration, login (with MFA support), logout, token lifecycle, and password management.
//!
//! # Login flow (with optional MFA)
//!
//! ```text
//! POST /v1/api/auth/login
//!   → verify credentials (pgcrypt)
//!   → if MFA enabled: return MfaChallengeResponse (short-lived PASETO challenge token)
//!   → else: create session row, return BasicLoginResponse (access + refresh tokens)
//!
//! POST /v1/api/auth/mfa/verify  (if MFA was required)
//!   → validate challenge token (no DB)
//!   → verify TOTP code (or recovery code)
//!   → create session row, return BasicLoginResponse
//! ```
//!
//! # Registration flow
//!
//! All DB writes happen inside a single transaction. Email verification is sent
//! *after* the transaction commits so a failed email send never rolls back registration.
//!
//! # Refresh token flow
//!
//! The refresh exchange runs inside a transaction:
//! 1. Validate the raw refresh token (DB lookup by SHA-256 hash).
//! 2. Generate a new access + refresh token pair for the *same* session ID.
//! 3. Rotate the session (replace hash, reset expiry, update `last_used_at`).

use crate::{
    http::api::dtos::{
        AuthContextResponse, AuthenticateRequest, AuthenticatedUserContext, BasicAuthLoginRequest,
        BasicLoginResponse, LoginOutcome, MfaChallengeResponse, RefreshTokenRequest,
        RegisterUserRequest, RegisterUserResponse,
    },
    ApiTokenService, EmailService, EmailVerificationService, IdentityProvider, IdentityService,
    MfaTotpService, PasswordResetService, PostgresClient, RoleType, ServiceError, SessionService,
    Sessions, User, UserIdentity, UserRole, UserRoleService, UserService, UserStatus,
};
use chrono::Utc;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthApplication {
    pg_client: Arc<PostgresClient>,
    identity_service: Arc<IdentityService>,
    user_service: Arc<UserService>,
    user_role_service: Arc<UserRoleService>,
    session_service: Arc<SessionService>,
    mfa_totp_service: Arc<MfaTotpService>,
    api_token_service: Arc<ApiTokenService>,
    email_verification_service: Arc<EmailVerificationService>,
    email_service: Arc<EmailService>,
    password_reset_service: Arc<PasswordResetService>,
}

impl AuthApplication {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pg_client: Arc<PostgresClient>,
        identity_service: Arc<IdentityService>,
        user_service: Arc<UserService>,
        user_role_service: Arc<UserRoleService>,
        session_service: Arc<SessionService>,
        mfa_totp_service: Arc<MfaTotpService>,
        api_token_service: Arc<ApiTokenService>,
        email_verification_service: Arc<EmailVerificationService>,
        email_service: Arc<EmailService>,
        password_reset_service: Arc<PasswordResetService>,
    ) -> Self {
        Self {
            pg_client,
            identity_service,
            user_service,
            user_role_service,
            session_service,
            mfa_totp_service,
            api_token_service,
            email_verification_service,
            email_service,
            password_reset_service,
        }
    }

    pub async fn authenticate_token(
        &self,
        request: AuthenticateRequest,
    ) -> Result<AuthContextResponse, ServiceError> {
        tracing::debug!("Authenticating token {}", request.access_token);
        let auth_context = self
            .session_service
            .authenticate_session_token(request.access_token.as_str())?;

        let roles = auth_context
            .roles
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let response = AuthContextResponse {
            user_id: auth_context.user_id,
            session_id: auth_context.session_id,
            roles,
            email_verified: auth_context.email_verified,
            must_change_password: auth_context.must_change_password,
            scope: auth_context.scope,
            policy_test_id: auth_context.policy_test_id,
        };

        Ok(response)
    }

    /// Validate a raw `sat_*` API token and return an auth context.
    /// Fetches the owning user's roles so the roles array is populated.
    /// session_id is set to Uuid::nil() — no session row exists for API token auth.
    pub async fn authenticate_api_token(
        &self,
        raw_token: &str,
    ) -> Result<AuthContextResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        let api_token = self
            .api_token_service
            .validate_token(&mut conn, raw_token)
            .await?;

        let user = self
            .user_service
            .find_user_by_id(&mut conn, api_token.user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::AuthenticationError("User not found".to_string()))?;

        let roles = self
            .user_role_service
            .get_user_roles(&mut conn, &user)
            .await?;

        let role_strings: Vec<String> =
            roles.iter().map(|r| r.type_.as_str().to_string()).collect();

        Ok(AuthContextResponse {
            user_id: api_token.user_id,
            session_id: Uuid::nil(),
            roles: role_strings,
            email_verified: true, // API tokens are trusted long-lived credentials
            must_change_password: false, // API token do not require password change
            scope: None,
            policy_test_id: None,
        })
    }

    pub async fn logout(&self, ctx: AuthenticatedUserContext) -> Result<(), ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        self.session_service
            .revoke_session(&mut conn, ctx.session_id, ctx.user_id)
            .await
    }

    pub async fn logout_all(&self, ctx: AuthenticatedUserContext) -> Result<usize, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        self.session_service
            .revoke_all_sessions(&mut conn, ctx.user_id)
            .await
    }

    pub async fn register_with_basic_auth(
        &self,
        request: RegisterUserRequest,
    ) -> Result<RegisterUserResponse, ServiceError> {
        tracing::debug!("Basic auth regisraion flow for email {}", request.email);
        // Get a connection from the pool
        let mut conn = self.pg_client.get_conn().await?;

        // Need to clone the repos because how rust uses this for future, need to ensure all
        // elements are available.
        let user_service = self.user_service.clone();
        let user_role_service = self.user_role_service.clone();
        let identity_service = self.identity_service.clone();
        let email_clone = request.email.clone();

        // Execute transaction — returns (response, identity_id) so we can trigger email after
        let (response, identity_id) = conn
            .transaction(move |trx| {
                let user_service = user_service.clone();
                let user_role_service = user_role_service.clone();
                let identity_service = identity_service.clone();
                let request = request;

                async move {
                    // Verify that email is available
                    identity_service
                        .verify_email_availability(trx, &request.email)
                        .await?;

                    // Create user entity
                    let user_id = Uuid::new_v4();
                    let new_user = User {
                        user_id,
                        first_name: Some(request.first_name.clone()),
                        last_name: Some(request.last_name.clone()),
                        avatar_url: request.avatar_url.clone(),
                        display_name: None,
                        status: UserStatus::PendingVerification,
                        token_version: 0,
                        mfa_required: false,
                        created_by: Some(user_id),
                        created_at: Some(Utc::now()),
                        updated_by: Some(user_id),
                        updated_at: Some(Utc::now()),
                    };
                    // Insert User
                    let persisted_user = user_service.create_user(trx, &new_user).await?;
                    // Create Identity entity
                    let identity_id = Uuid::new_v4();
                    let new_identity = UserIdentity {
                        identity_id,
                        user_id,
                        provider: IdentityProvider::EmailPassword,
                        provider_user_id: None,
                        email: request.email.clone(),
                        password_hash: Some(request.password.clone()),
                        password_changed_at: None,
                        email_verified: Some(false),
                        oauth_access_token: None,
                        oauth_refresh_token: None,
                        oauth_token_expires_at: None,
                        is_primary: false,
                        last_login_at: None,
                        must_change_password: false,
                        created_by: Some(user_id),
                        created_at: Some(Utc::now()),
                        updated_by: Some(user_id),
                        updated_at: Some(Utc::now()),
                    };

                    // insert identity
                    identity_service.create_identity(trx, &new_identity).await?;
                    let role = user_role_service
                        .get_role_by_type(trx, RoleType::User)
                        .await?;
                    let user_role = UserRole {
                        user_role_id: Uuid::new_v4(),
                        user_id: new_user.user_id,
                        role_id: role.role_id,
                        created_at: Utc::now(),
                        created_by: Some(new_user.user_id),
                    };
                    user_role_service.add_role_to_user(trx, &user_role).await?;
                    // add log message
                    tracing::debug!("User with email {} regisered successfully", request.email);
                    // Create and return User Response
                    let regiser_user_response = RegisterUserResponse {
                        user_id: persisted_user.user_id,
                        first_name: persisted_user.first_name.clone().unwrap(),
                        last_name: persisted_user.last_name.clone().unwrap(),
                        avatar_url: persisted_user.avatar_url.clone(),
                        status: persisted_user.status.clone(),
                    };

                    Ok::<(RegisterUserResponse, Uuid), ServiceError>((
                        regiser_user_response,
                        identity_id,
                    ))
                }
                .scope_boxed()
            })
            .await?;

        // After the transaction: create verification token and attempt to send email.
        // A failure here is non-fatal — we log it and still return the registration response.
        let first_name_clone = response.first_name.clone();
        let mut conn2 = self.pg_client.get_conn().await?;
        match self
            .email_verification_service
            .create_verification(&mut conn2, identity_id, response.user_id)
            .await
        {
            Ok(raw_token) => {
                let _ = self
                    .email_service
                    .send_verification_email(
                        &mut conn2,
                        &email_clone,
                        &first_name_clone,
                        &raw_token,
                    )
                    .await
                    .map_err(|e| {
                        tracing::warn!("Verification email send failed: {e}");
                    });
            }
            Err(e) => {
                tracing::warn!("Failed to create verification token: {e}");
            }
        }

        Ok(response)
    }

    pub async fn verify_email(&self, raw_token: String) -> Result<(), ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        let identity_id = self
            .email_verification_service
            .consume_token(&mut conn, &raw_token)
            .await?;
        self.identity_service
            .mark_email_verified(&mut conn, identity_id)
            .await?;
        // Promote user from PendingVerification → Active after email confirmed
        if let Some(identity) = self
            .identity_service
            .find_identity_by_id(&mut conn, identity_id)
            .await?
        {
            if let Some(user) = self
                .user_service
                .find_user_by_id(&mut conn, identity.user_id)
                .await?
            {
                if user.status == UserStatus::PendingVerification {
                    self.user_service
                        .update_user_status(&mut conn, identity.user_id, UserStatus::Active)
                        .await?;
                }
            }
        }
        Ok(())
    }

    /// POST /v1/api/auth/token/exchange
    ///
    /// Exchange an admin API token for a PASETO session token pair for a target user.
    /// The API token is an admin-level integration credential — the email, display_name,
    /// and avatar_url in the request body identify the target user whose session will be created.
    ///
    /// If the user does not exist, a new user is created with:
    /// - provider = token_federation
    /// - password = null
    /// - email_verified = true
    /// - status = active
    /// - must_change_password = false
    /// - role = user (default)
    ///
    /// If the user exists with provider=token_federation, their display_name and avatar_url
    /// are updated from the request.
    ///
    /// On success, returns session tokens. MFA is skipped (API token is already a strong credential).
    pub async fn exchange_api_token_for_session(
        &self,
        raw_api_token: String,
        email: String,
        display_name: String,
        avatar_url: Option<String>,
    ) -> Result<BasicLoginResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        let identity_service = self.identity_service.clone();
        let user_service = self.user_service.clone();
        let user_role_service = self.user_role_service.clone();
        let session_service = self.session_service.clone();
        let api_token_service = self.api_token_service.clone();

        let response = conn
            .transaction(move |trx| {
                let identity_service = identity_service.clone();
                let user_service = user_service.clone();
                let user_role_service = user_role_service.clone();
                let session_service = session_service.clone();
                let api_token_service = api_token_service.clone();
                let email = email;
                let display_name = display_name;
                let avatar_url = avatar_url;

                async move {
                    // 1. Validate the API token (checks hash, revoked_at, expires_at)
                    let api_token = api_token_service
                        .validate_token(trx, &raw_api_token)
                        .await?;
                    let admin_user_id = api_token.user_id;

                    // 1b. Verify the token owner has admin role
                    let admin_roles = user_role_service
                        .get_roles_by_user_id(trx, admin_user_id)
                        .await?;
                    let is_admin = admin_roles.iter().any(|r| r.type_ == RoleType::Admin);
                    if !is_admin {
                        return Err(ServiceError::AuthorizationError(
                            "Insufficient permissions".to_string(),
                        ));
                    }

                    // 2. Find the target user by email from the request body
                    let identity = identity_service.find_identity_by_email(trx, &email).await?;

                    let (user, updated_identity) = if let Some(existing_identity) = identity {
                        // User exists - validate and update if needed
                        let user = user_service
                            .find_user_by_id(trx, existing_identity.user_id)
                            .await?
                            .ok_or_else(|| {
                                ServiceError::AuthenticationError("User not found".to_string())
                            })?;

                        // Validate user state for token federation
                        if existing_identity.provider != IdentityProvider::TokenFederation {
                            return Err(ServiceError::ValidationError(
                                "User exists but was not created via token federation".to_string(),
                            ));
                        }

                        // Update display_name and avatar_url on the user record
                        let updated_user = user_service
                            .update_federated_user_profile(
                                trx,
                                user.user_id,
                                display_name.clone(),
                                avatar_url.clone(),
                            )
                            .await?;

                        // Ensure identity has email_verified = true and must_change_password = false
                        let identity_updated = identity_service
                            .ensure_federation_identity_state(trx, existing_identity.identity_id)
                            .await?;

                        (updated_user, identity_updated)
                    } else {
                        // User does not exist - create new federated user
                        let now = Utc::now();
                        let user_id = Uuid::new_v4();

                        // Create user with display_name and avatar_url
                        // Split display_name into first_name/last_name for compatibility
                        let name_parts: Vec<&str> = display_name.splitn(2, ' ').collect();
                        let first_name = name_parts.first().map(|s| s.to_string());
                        let last_name = name_parts.get(1).map(|s| s.to_string());

                        let new_user = User {
                            user_id,
                            first_name,
                            last_name,
                            avatar_url: avatar_url.clone(),
                            display_name: Some(display_name.clone()),
                            status: UserStatus::Active,
                            token_version: 0,
                            mfa_required: false,
                            created_by: Some(admin_user_id),
                            created_at: Some(now),
                            updated_by: Some(admin_user_id),
                            updated_at: Some(now),
                        };
                        let persisted_user = user_service.create_user(trx, &new_user).await?;

                        // Create identity with provider=token_federation, password=null, email_verified=true
                        let identity_id = Uuid::new_v4();
                        let new_identity = UserIdentity {
                            identity_id,
                            user_id,
                            provider: IdentityProvider::TokenFederation,
                            provider_user_id: None,
                            email: email.clone(),
                            password_hash: None,
                            password_changed_at: None,
                            email_verified: Some(true),
                            oauth_access_token: None,
                            oauth_refresh_token: None,
                            oauth_token_expires_at: None,
                            is_primary: true,
                            last_login_at: None,
                            must_change_password: false,
                            created_by: Some(admin_user_id),
                            created_at: Some(now),
                            updated_by: Some(admin_user_id),
                            updated_at: Some(now),
                        };
                        let persisted_identity =
                            identity_service.create_identity(trx, &new_identity).await?;

                        // Assign default 'user' role
                        let user_role = user_role_service
                            .get_role_by_type(trx, RoleType::User)
                            .await?;
                        let new_user_role = UserRole {
                            user_role_id: Uuid::new_v4(),
                            user_id,
                            role_id: user_role.role_id,
                            created_by: Some(admin_user_id),
                            created_at: Utc::now(),
                        };
                        user_role_service
                            .add_role_to_user(trx, &new_user_role)
                            .await?;

                        (persisted_user, persisted_identity)
                    };

                    // 3. Get user roles
                    let roles = user_role_service.get_user_roles(trx, &user).await?;

                    // 4. Generate session tokens
                    let session_id = uuid::Uuid::new_v4();
                    let tokens = session_service.generate_session_token(
                        &user,
                        &session_id,
                        &roles,
                        &updated_identity,
                    )?;

                    // 5. Create session row for the target user
                    let now = chrono::Utc::now();
                    let access_expires_at = now + session_service.access_ttl;
                    let refresh_expires_at = now + session_service.refresh_ttl;

                    let new_session = Sessions {
                        session_id,
                        user_id: user.user_id,
                        identity_id: updated_identity.identity_id,
                        refresh_token_hash: tokens.refresh_token_hash.clone(),
                        refresh_token_family: uuid::Uuid::new_v4(),
                        refresh_token_expires_at: refresh_expires_at,
                        user_agent: None,
                        ip_address: None,
                        device_type: None,
                        revoked_at: None,
                        revoked_reason: None,
                        last_used_at: None,
                        created_at: Some(now),
                        updated_at: Some(now),
                        created_by: Some(admin_user_id),
                        updated_by: Some(admin_user_id),
                    };

                    session_service.create_session(trx, &new_session).await?;

                    // 6. Record API token usage (last_used_at)
                    api_token_service
                        .record_usage(trx, api_token.api_token_id)
                        .await?;

                    Ok::<BasicLoginResponse, ServiceError>(BasicLoginResponse {
                        user_id: user.user_id,
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                        expires_at: access_expires_at,
                        must_change_password: updated_identity.must_change_password,
                        mfa_setup_required: false,
                    })
                }
                .scope_boxed()
            })
            .await?;

        Ok(response)
    }

    pub async fn resend_verification(&self, email: String) -> Result<(), ServiceError> {
        tracing::info!(email, "resend_verification: starting");

        let mut conn = self.pg_client.get_conn().await.map_err(|e| {
            tracing::error!(error = ?e, "resend_verification: failed to acquire DB connection");
            e
        })?;
        tracing::debug!("resend_verification: DB connection acquired");

        let identity = self
            .identity_service
            .find_identity_by_email(&mut conn, &email)
            .await
            .map_err(|e| {
                tracing::error!(email, error = ?e, "resend_verification: error looking up identity by email");
                e
            })?
            .ok_or_else(|| {
                tracing::warn!(email, "resend_verification: no identity found for email");
                ServiceError::NotFoundError("Email not found".to_string())
            })?;
        tracing::debug!(
            identity_id = %identity.identity_id,
            email_verified = ?identity.email_verified,
            "resend_verification: identity found"
        );

        if identity.email_verified.unwrap_or(false) {
            tracing::info!(
                email,
                "resend_verification: email already verified, skipping"
            );
            return Ok(()); // already verified — silently succeed
        }

        let user = self
            .user_service
            .find_user_by_id(&mut conn, identity.user_id)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %identity.user_id, error = ?e, "resend_verification: error looking up user");
                e
            })?
            .ok_or_else(|| {
                tracing::error!(user_id = %identity.user_id, "resend_verification: user row missing for identity");
                ServiceError::InternalError("User not found".into())
            })?;
        let first_name = user.first_name.unwrap_or_default();
        tracing::debug!(first_name, "resend_verification: user found");

        tracing::debug!(identity_id = %identity.identity_id, "resend_verification: creating verification token");
        let raw_token = self
            .email_verification_service
            .create_verification(&mut conn, identity.identity_id, identity.user_id)
            .await
            .map_err(|e| {
                tracing::error!(identity_id = %identity.identity_id, error = ?e, "resend_verification: failed to create verification token");
                e
            })?;
        tracing::debug!(
            "resend_verification: verification token created, handing off to email service"
        );

        self.email_service
            .send_verification_email(&mut conn, &email, &first_name, &raw_token)
            .await
    }

    /// POST /v1/api/auth/password/forgot
    /// Always returns Ok(()) regardless of whether the email exists (anti-enumeration).
    pub async fn forgot_password(&self, email: String) -> Result<(), ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        let identity = match self
            .identity_service
            .find_identity_by_email(&mut conn, &email)
            .await?
        {
            Some(i) => i,
            None => {
                tracing::debug!("Password reset for unknown email silently ignored");
                return Ok(());
            }
        };
        let user = self
            .user_service
            .find_user_by_id(&mut conn, identity.user_id)
            .await?
            .ok_or_else(|| ServiceError::InternalError("User not found".into()))?;
        let first_name = user.first_name.unwrap_or_default();
        let raw_token = self
            .password_reset_service
            .create_reset_token(&mut conn, identity.identity_id, identity.user_id)
            .await?;
        let _ = self
            .email_service
            .send_password_reset_email(&mut conn, &identity.email, &first_name, &raw_token)
            .await
            .map_err(|e| tracing::warn!("Password reset email failed: {e}"));
        Ok(())
    }

    /// POST /v1/api/auth/password/reset
    pub async fn reset_password(
        &self,
        raw_token: String,
        new_password: String,
    ) -> Result<(), ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        let identity_id = self
            .password_reset_service
            .consume_token(&mut conn, &raw_token)
            .await?;
        let identity = self
            .identity_service
            .find_identity_by_id(&mut conn, identity_id)
            .await?
            .ok_or_else(|| {
                ServiceError::InternalError("Identity not found after token consume".into())
            })?;
        self.identity_service
            .update_password(&mut conn, identity_id, new_password)
            .await?;
        self.session_service
            .revoke_all_sessions(&mut conn, identity.user_id)
            .await?;
        let user = self
            .user_service
            .find_user_by_id(&mut conn, identity.user_id)
            .await?
            .ok_or_else(|| ServiceError::InternalError("User not found".into()))?;
        let first_name = user.first_name.unwrap_or_default();
        let _ = self
            .email_service
            .send_password_changed_email(&mut conn, &identity.email, &first_name)
            .await
            .map_err(|e| tracing::warn!("Password changed notification failed: {e}"));
        Ok(())
    }
    pub async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> Result<BasicLoginResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        let identity_service = self.identity_service.clone();
        let user_service = self.user_service.clone();
        let user_role_service = self.user_role_service.clone();
        let session_service = self.session_service.clone();

        let response = conn
            .transaction(move |trx| {
                let identity_service = identity_service.clone();
                let user_service = user_service.clone();
                let user_role_service = user_role_service.clone();
                let session_service = session_service.clone();
                let request = request;

                async move {
                    // 1. Validate the refresh token and retrieve the session
                    let session = session_service
                        .validate_refresh_token(trx, &request.refresh_token)
                        .await?;

                    // 2. Fetch user + identity + roles (needed to sign new PASETO token)
                    let user = user_service
                        .find_user_by_id(trx, session.user_id)
                        .await
                        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                        .ok_or_else(|| {
                            ServiceError::AuthenticationError("User not found".to_string())
                        })?;

                    let identity = identity_service
                        .find_primary_identity_by_user_id(trx, session.user_id)
                        .await?
                        .ok_or_else(|| {
                            ServiceError::AuthenticationError("Identity not found".to_string())
                        })?;

                    let roles = user_role_service.get_user_roles(trx, &user).await?;

                    // 3. Generate new token pair (same session_id — no new session row)
                    let new_tokens = session_service.generate_session_token(
                        &user,
                        &session.session_id,
                        &roles,
                        &identity,
                    )?;

                    // 4. Rotate: store new hash, reset expiry, update last_used_at
                    session_service
                        .rotate_session(
                            trx,
                            session.session_id,
                            new_tokens.refresh_token_hash.clone(),
                        )
                        .await?;

                    let expires_at = chrono::Utc::now() + session_service.access_ttl;

                    Ok::<BasicLoginResponse, ServiceError>(BasicLoginResponse {
                        user_id: user.user_id,
                        access_token: new_tokens.access_token,
                        refresh_token: new_tokens.refresh_token,
                        expires_at,
                        must_change_password: identity.must_change_password,
                        mfa_setup_required: false,
                    })
                }
                .scope_boxed()
            })
            .await?;

        Ok(response)
    }

    pub async fn basic_auth_login(
        &self,
        request: BasicAuthLoginRequest,
    ) -> Result<LoginOutcome, ServiceError> {
        tracing::debug!("Basic auth login credentials {:#?}", request);
        // Get db connection
        let mut conn = self.pg_client.get_conn().await?;

        let identity_service = self.identity_service.clone();
        let user_service = self.user_service.clone();
        let user_role_service = self.user_role_service.clone();
        let session_service = self.session_service.clone();
        let mfa_totp_service = self.mfa_totp_service.clone();

        let response = conn
            .transaction(move |trx| {
                let identity_service = identity_service.clone();
                let user_service = user_service.clone();
                let user_role_service = user_role_service.clone();
                let session_service = session_service.clone();
                let mfa_totp_service = mfa_totp_service.clone();
                let request = request;

                async move {
                    // Verify identity (credentials)
                    let found_identity = identity_service
                        .verify_identity_exists(
                            trx,
                            request.email.as_str(),
                            request.password.as_str(),
                        )
                        .await?;

                    // Fetch user
                    let user = user_service
                        .find_user_by_id(trx, found_identity.user_id)
                        .await
                        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                        .ok_or_else(|| {
                            ServiceError::AuthenticationError("USER_NOT_FOUND".to_string())
                        })?;

                    // MFA gate: if TOTP enrolled and enabled, issue a challenge token
                    let mfa_enabled = mfa_totp_service.is_mfa_enabled(trx, user.user_id).await?;
                    if mfa_enabled {
                        let mfa_token =
                            session_service.generate_mfa_challenge_token(user.user_id)?;
                        return Ok(LoginOutcome::MfaChallenge(MfaChallengeResponse {
                            user_id: user.user_id,
                            mfa_required: true,
                            mfa_session_token: mfa_token,
                        }));
                    }
                    // If admin mandated MFA but user hasn't enrolled yet, flag for setup
                    let mfa_setup_required = user.mfa_required && !mfa_enabled;

                    // Fetch user roles
                    let user_roles = user_role_service.get_user_roles(trx, &user).await?;

                    // Create session
                    let session_id = uuid::Uuid::new_v4();
                    let refresh_token_family = uuid::Uuid::new_v4();
                    let tokens = session_service.generate_session_token(
                        &user,
                        &session_id,
                        &user_roles,
                        &found_identity,
                    )?;
                    // tokens: { access_token, refresh_token, refresh_token_hash }

                    // Decide TTLs (keep consistent with your SessionService access ttl)
                    let now = chrono::Utc::now();
                    let access_expires_at = now + session_service.access_ttl;
                    let refresh_expires_at = now + session_service.refresh_ttl;

                    let refresh_token_hash = tokens.refresh_token_hash.clone();
                    let access_token = tokens.access_token.clone();
                    let refresh_token = tokens.refresh_token.clone();
                    // Build session object
                    let new_session = Sessions {
                        session_id,
                        user_id: user.user_id,
                        identity_id: found_identity.identity_id,
                        refresh_token_hash,
                        refresh_token_family,
                        refresh_token_expires_at: refresh_expires_at,
                        user_agent: None,
                        ip_address: None,
                        device_type: None,
                        revoked_at: None,
                        revoked_reason: None,
                        last_used_at: None,
                        created_at: Some(Utc::now()),
                        updated_at: Some(Utc::now()),
                        created_by: Some(user.user_id),
                        updated_by: Some(user.user_id),
                    };

                    session_service
                        .create_session(trx, &new_session)
                        .await
                        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

                    tracing::info!("Valid Credentials for email: {}", found_identity.email);

                    // Return response
                    let response = BasicLoginResponse {
                        user_id: user.user_id,
                        access_token,
                        refresh_token,
                        expires_at: access_expires_at,
                        must_change_password: found_identity.must_change_password,
                        mfa_setup_required,
                    };
                    Ok::<LoginOutcome, ServiceError>(LoginOutcome::Success(response))
                }
                .scope_boxed()
            })
            .await?;

        Ok(response)
    }
}
