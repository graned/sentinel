//! MFA application layer — orchestrates TOTP enrollment and MFA login verification.
//!
//! # Enrollment flow (two-step)
//!
//! 1. `start_enrollment`   — generates a TOTP secret + `otpauth://` URI; returns it to the client.
//! 2. `confirm_enrollment` — verifies the first code from the authenticator app,
//!    enables MFA for the user, and returns 8 one-time recovery codes.
//!
//! # MFA login flow
//!
//! After a successful password login, `basic_auth_login` returns a short-lived PASETO
//! MFA challenge token (`sub = "Sentinel:MfaChallenge"`).  The client submits that token
//! plus a TOTP code to `verify_mfa_login`, which:
//!
//! 1. Validates the challenge token (pure crypto, no DB).
//! 2. Checks per-token attempt rate limit (in-memory, SHA-256-keyed).
//! 3. Inside a transaction: verifies TOTP code (or recovery code), fetches user/identity/roles,
//!    creates a session row, and returns full access + refresh tokens.
//! 4. On success: clears the attempt counter. On `MfaInvalidCode`: increments it.
//!
//! # Brute-force protection
//!
//! `mfa_attempts` is an in-memory `HashMap<SHA-256(token), MfaAttemptEntry>` protected by
//! a `Mutex`. After 5 failed attempts within a 15-minute window, the endpoint returns
//! `429 MFA_ATTEMPT_LIMIT_EXCEEDED`. Entries are pruned on every call.

use crate::{
    http::api::dtos::{
        BasicLoginResponse, ConfirmMfaEnrollmentResponse, StartMfaEnrollmentResponse,
    },
    IdentityService, MfaTotpService, PostgresClient, ServiceError, SessionService, Sessions,
    UserRoleService, UserService,
};
use chrono::Utc;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Maximum failed TOTP attempts allowed within the rate-limit window.
const MAX_MFA_ATTEMPTS: u8 = 5;
/// Rate-limit window duration in seconds (15 minutes).
const MFA_WINDOW_SECS: u64 = 900;

/// Tracks failed MFA attempts for a single challenge token within a time window.
/// Stored in the `mfa_attempts` map keyed by `SHA-256(mfa_session_token)`.
struct MfaAttemptEntry {
    /// Number of failed attempts within the current window.
    count: u8,
    /// When the current window started (used to detect window expiry).
    window_start: Instant,
}

pub struct MfaApplication {
    pg_client: Arc<PostgresClient>,
    mfa_totp_service: Arc<MfaTotpService>,
    session_service: Arc<SessionService>,
    user_service: Arc<UserService>,
    identity_service: Arc<IdentityService>,
    user_role_service: Arc<UserRoleService>,
    mfa_attempts: Arc<Mutex<HashMap<String, MfaAttemptEntry>>>,
}

impl MfaApplication {
    pub fn new(
        pg_client: Arc<PostgresClient>,
        mfa_totp_service: Arc<MfaTotpService>,
        session_service: Arc<SessionService>,
        user_service: Arc<UserService>,
        identity_service: Arc<IdentityService>,
        user_role_service: Arc<UserRoleService>,
    ) -> Self {
        Self {
            pg_client,
            mfa_totp_service,
            session_service,
            user_service,
            identity_service,
            user_role_service,
            mfa_attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Hash the MFA session token for use as a map key (avoid storing raw tokens in memory).
    fn token_key(token: &str) -> String {
        format!("{:x}", Sha256::digest(token.as_bytes()))
    }

    /// Start TOTP enrollment for an authenticated user.
    /// Returns an otpauth URI to be scanned with an authenticator app.
    pub async fn start_enrollment(
        &self,
        user_id: Uuid,
    ) -> Result<StartMfaEnrollmentResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        // Fetch the user's email for the otpauth URI label
        let identity = self
            .identity_service
            .find_primary_identity_by_user_id(&mut conn, user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Identity not found".to_string()))?;

        let data = self
            .mfa_totp_service
            .start_enrollment(&mut conn, user_id, &identity.email)
            .await?;

        Ok(StartMfaEnrollmentResponse {
            otpauth_uri: data.otpauth_uri,
        })
    }

    /// Confirm TOTP enrollment by verifying the first code.
    /// Returns one-time recovery codes on success.
    pub async fn confirm_enrollment(
        &self,
        user_id: Uuid,
        code: String,
    ) -> Result<ConfirmMfaEnrollmentResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        let data = self
            .mfa_totp_service
            .confirm_enrollment(&mut conn, user_id, &code)
            .await?;

        Ok(ConfirmMfaEnrollmentResponse {
            recovery_codes: data.recovery_codes,
        })
    }

    /// Exchange an MFA challenge token + TOTP code for a full session.
    pub async fn verify_mfa_login(
        &self,
        mfa_session_token: String,
        code: String,
    ) -> Result<BasicLoginResponse, ServiceError> {
        // Validate the MFA challenge token (outside the transaction)
        let user_id = self
            .session_service
            .verify_mfa_challenge_token(&mfa_session_token)?;

        // Check per-token attempt limit (prevents distributed brute-force)
        let key = Self::token_key(&mfa_session_token);
        {
            let mut attempts = self.mfa_attempts.lock().await;

            // Prune stale entries
            attempts.retain(|_, v| v.window_start.elapsed().as_secs() < MFA_WINDOW_SECS);

            if let Some(entry) = attempts.get(&key) {
                if entry.window_start.elapsed().as_secs() < MFA_WINDOW_SECS
                    && entry.count >= MAX_MFA_ATTEMPTS
                {
                    return Err(ServiceError::MfaAttemptLimitExceeded(
                        "Too many MFA attempts. Try again later.".to_string(),
                    ));
                }
            }
        }

        let mut conn = self.pg_client.get_conn().await?;

        let identity_service = self.identity_service.clone();
        let user_service = self.user_service.clone();
        let user_role_service = self.user_role_service.clone();
        let session_service = self.session_service.clone();
        let mfa_totp_service = self.mfa_totp_service.clone();
        let mfa_attempts = self.mfa_attempts.clone();

        let result = conn
            .transaction(move |mut trx| {
                let identity_service = identity_service.clone();
                let user_service = user_service.clone();
                let user_role_service = user_role_service.clone();
                let session_service = session_service.clone();
                let mfa_totp_service = mfa_totp_service.clone();
                let code = code.clone();

                async move {
                    // Verify TOTP code (or recovery code)
                    mfa_totp_service.verify(&mut trx, user_id, &code).await?;

                    // Fetch user
                    let user = user_service
                        .find_user_by_id(&mut trx, user_id)
                        .await
                        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                        .ok_or_else(|| {
                            ServiceError::AuthenticationError("User not found".to_string())
                        })?;

                    // Fetch identity (for session token claims)
                    let identity = identity_service
                        .find_primary_identity_by_user_id(&mut trx, user_id)
                        .await?
                        .ok_or_else(|| {
                            ServiceError::AuthenticationError("Identity not found".to_string())
                        })?;

                    // Fetch roles
                    let user_roles = user_role_service.get_user_roles(&mut trx, &user).await?;

                    // Generate tokens
                    let session_id = uuid::Uuid::new_v4();
                    let refresh_token_family = uuid::Uuid::new_v4();
                    let tokens = session_service.generate_session_token(
                        &user,
                        &session_id,
                        &user_roles,
                        &identity,
                    )?;

                    let now = Utc::now();
                    let access_expires_at = now + session_service.access_ttl;
                    let refresh_expires_at = now + session_service.refresh_ttl;

                    let new_session = Sessions {
                        session_id,
                        user_id: user.user_id,
                        identity_id: identity.identity_id,
                        refresh_token_hash: tokens.refresh_token_hash.clone(),
                        refresh_token_family,
                        refresh_token_expires_at: refresh_expires_at,
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

                    session_service
                        .create_session(&mut trx, &new_session)
                        .await
                        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

                    Ok::<BasicLoginResponse, ServiceError>(BasicLoginResponse {
                        user_id: user.user_id,
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                        expires_at: access_expires_at,
                        must_change_password: identity.must_change_password,
                        mfa_setup_required: false,
                    })
                }
                .scope_boxed()
            })
            .await;

        // Track attempt outcomes outside the transaction to avoid holding locks inside async DB ops
        match &result {
            Ok(_) => {
                // Successful login — clear attempt counter for this token
                mfa_attempts.lock().await.remove(&key);
            }
            Err(ServiceError::MfaInvalidCode(_)) => {
                // Failed code — increment attempt counter
                let mut attempts = mfa_attempts.lock().await;
                let entry = attempts.entry(key).or_insert(MfaAttemptEntry {
                    count: 0,
                    window_start: Instant::now(),
                });
                if entry.window_start.elapsed().as_secs() >= MFA_WINDOW_SECS {
                    entry.count = 0;
                    entry.window_start = Instant::now();
                }
                entry.count = entry.count.saturating_add(1);
            }
            Err(_) => {}
        }

        Ok(result?)
    }
}
