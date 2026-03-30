//! User password application — authenticated password-change flow.
//!
//! Handles `POST /user/password/change` for users who are already logged in and want
//! to change their own password.  Distinct from the public password-reset flow
//! (forgotten password → reset link) handled by `AuthApplication`.
//!
//! # Steps
//!
//! 1. Look up the user's identity and verify the current password via `crypt()`.
//! 2. Hash the new password using `crypt(new_password, gen_salt('bf'))`.
//! 3. Update `user_identities.password_hash` and clear `must_change_password`.
//! 4. Revoke all active sessions (forces re-login everywhere).
//! 5. Send a "password changed" notification email (async, non-fatal if SMTP not configured).

use crate::{
    http::api::dtos::AuthenticatedUserContext, EmailService, IdentityService, PostgresClient,
    ServiceError, SessionService, UserService,
};
use std::sync::Arc;

pub struct UserPasswordApplication {
    pg_client: Arc<PostgresClient>,
    identity_service: Arc<IdentityService>,
    session_service: Arc<SessionService>,
    user_service: Arc<UserService>,
    email_service: Arc<EmailService>,
}

impl UserPasswordApplication {
    pub fn new(
        pg_client: Arc<PostgresClient>,
        identity_service: Arc<IdentityService>,
        session_service: Arc<SessionService>,
        user_service: Arc<UserService>,
        email_service: Arc<EmailService>,
    ) -> Self {
        Self {
            pg_client,
            identity_service,
            session_service,
            user_service,
            email_service,
        }
    }

    /// POST /v1/api/user/password/change — requires authentication.
    /// Verifies the current password, updates to the new password, revokes all sessions,
    /// and sends a "password changed" notification email.
    pub async fn change_password(
        &self,
        ctx: AuthenticatedUserContext,
        current_password: String,
        new_password: String,
    ) -> Result<(), ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        let identity = self
            .identity_service
            .find_primary_identity_by_user_id(&mut conn, ctx.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Identity not found".into()))?;

        // Verify current password using the existing credentials check
        self.identity_service
            .verify_identity_exists(&mut conn, &identity.email, &current_password)
            .await
            .map_err(|_| {
                ServiceError::AuthenticationError("Current password is incorrect".into())
            })?;

        self.identity_service
            .update_password(&mut conn, identity.identity_id, new_password)
            .await?;

        self.session_service
            .revoke_all_sessions(&mut conn, ctx.user_id)
            .await?;

        let user = self
            .user_service
            .find_user_by_id(&mut conn, ctx.user_id)
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
}
