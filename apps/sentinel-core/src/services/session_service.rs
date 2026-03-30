//! Session management: PASETO token issuance, validation, rotation, and revocation.
//!
//! # Token format
//!
//! Sentinel uses two token types:
//!
//! | Token | Format | Storage |
//! |-------|--------|---------|
//! | Access token | PASETO v4.local (encrypted symmetric) | Never stored; validated in-memory |
//! | Refresh token | `rt_<base64url>` (512 random bits) | Only the SHA-256 hash stored in `sessions` |
//!
//! ## Access token claims
//!
//! | Claim | Type | Description |
//! |-------|------|-------------|
//! | `sub` | `"Sentinel:Session"` | Identifies the token type |
//! | `sid` | UUID | Session ID — used to look up the `sessions` row for revocation checks |
//! | `uid` | UUID | User ID |
//! | `usr` | JSON object | `{ email, first_name, last_name }` |
//! | `roles` | JSON array | User role strings (e.g. `["admin"]`) |
//! | `ev`  | bool | Email verified status (baked in at login — re-login required after verification) |
//! | `mcp` | bool | Must change password flag |
//! | `exp` | RFC3339 | Expiry timestamp (default: 5 minutes from issuance) |
//!
//! # Revocation
//!
//! PASETO tokens remain cryptographically valid after logout — there is no token
//! blacklist. Revocation is tracked via `sessions.revoked_at IS NOT NULL` (soft-delete).
//! Middleware that needs strong revocation guarantees should look up the session row;
//! the current middleware trusts the token directly for performance.

use crate::{
    DbConnection, RevocationReason, Role, ServiceError, SessionRepository, Sessions, User,
    UserIdentity,
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, Utc};
use rand::{rngs::OsRng, RngCore};
use rusty_paseto::prelude::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use std::sync::Arc;
use uuid::Uuid;

/// Bundle of raw tokens returned by [`SessionService::generate_session_token`].
///
/// The `access_token` is returned directly to the client.
/// The `refresh_token` is returned to the client **once**.
/// Only `refresh_token_hash` (SHA-256 of the raw token) is persisted in the DB.
pub struct SessionTokens {
    pub access_token: String,
    pub refresh_token: String,
    /// SHA-256 hex digest of the raw refresh token — stored in `sessions.refresh_token_hash`.
    pub refresh_token_hash: String,
}

/// Decoded claims from a validated PASETO access token.
///
/// Populated by [`SessionService::authenticate_session_token`] and inserted
/// into Axum request extensions by `authenticate_middleware` so that handlers
/// and downstream middleware can access the caller's identity without re-parsing
/// the token.
#[derive(Debug)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    /// Raw JSON array of role strings (e.g. `["admin", "user"]`).
    pub roles: Value,
    /// Whether the user's email address has been verified.
    /// This is baked into the token at login time — users must re-login after verifying.
    pub email_verified: bool,
    /// Whether the user must change their password before accessing protected endpoints.
    pub must_change_password: bool,
    /// Present only for policy-test tokens (`"policy_test"` scope).
    pub scope: Option<String>,
    /// Policy ID embedded in policy-test tokens. `None` for regular session tokens.
    pub policy_test_id: Option<Uuid>,
}

/// Diesel changeset for soft-revoking a session (sets `revoked_at` + reason).
/// Private to this module — not exposed to the application layer.
#[derive(diesel::AsChangeset)]
#[diesel(table_name = crate::schema::sessions)]
struct SessionRevocationChangeset {
    revoked_at: Option<DateTime<Utc>>,
    revoked_reason: Option<RevocationReason>,
}

/// Diesel changeset for rotating a refresh token (replaces hash + resets expiry).
/// Private to this module — not exposed to the application layer.
#[derive(diesel::AsChangeset)]
#[diesel(table_name = crate::schema::sessions)]
struct SessionRotationChangeset {
    refresh_token_hash: String,
    refresh_token_expires_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
}

/// Owns all PASETO token logic (issuance, parsing, MFA challenge tokens, test tokens)
/// and session lifecycle management (creation, rotation, revocation).
///
/// The symmetric encryption key is stored as a `PasetoSymmetricKey<V4, Local>` so tokens
/// are encrypted (not just signed) — their contents are opaque to clients.
pub struct SessionService {
    session_repository: Arc<SessionRepository>,
    /// PASETO v4.local symmetric key derived from the 32-byte `HEX_KEY` env var.
    key: PasetoSymmetricKey<V4, Local>,
    /// How long access tokens are valid. Default: 5 minutes.
    pub access_ttl: Duration,
    /// How long refresh tokens are valid. Default: 30 days.
    pub refresh_ttl: Duration,
    /// Number of random bytes for each refresh token. Default: 64 (512 bits).
    refresh_bytes: usize,
}

impl SessionService {
    /// Construct a new `SessionService` from a 32-byte raw key.
    /// Default TTLs: access = 5 min, refresh = 30 days.
    pub fn new(session_repository: Arc<SessionRepository>, key_32: [u8; 32]) -> Self {
        let key = PasetoSymmetricKey::<V4, Local>::from(Key::from(&key_32));

        Self {
            session_repository,
            key,
            access_ttl: Duration::minutes(5),
            refresh_ttl: Duration::days(30),
            refresh_bytes: 64, // 512-bit refresh token
        }
    }

    /// Persist a new session row created by the application layer.
    pub async fn create_session(
        &self,
        conn: &mut DbConnection<'_>,
        session: &Sessions,
    ) -> Result<Sessions, ServiceError> {
        self.session_repository
            .create(conn, session)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Soft-revoke a single session owned by `user_id` (user logout).
    /// Returns `AuthorizationError` if the session belongs to a different user.
    pub async fn revoke_session(
        &self,
        conn: &mut DbConnection<'_>,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        let session = self
            .session_repository
            .find_by_id(conn, session_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFoundError("Session not found".to_string()))?;

        if session.user_id != user_id {
            return Err(ServiceError::AuthorizationError(
                "Cannot revoke another user's session".to_string(),
            ));
        }

        let changeset = SessionRevocationChangeset {
            revoked_at: Some(Utc::now()),
            revoked_reason: Some(RevocationReason::UserLogout),
        };

        self.session_repository
            .update(conn, session_id, changeset)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Soft-revoke all active sessions for `user_id` (logout-all / password reset).
    /// Returns the number of sessions that were revoked.
    pub async fn revoke_all_sessions(
        &self,
        conn: &mut DbConnection<'_>,
        user_id: Uuid,
    ) -> Result<usize, ServiceError> {
        self.session_repository
            .revoke_all_active_sessions_for_user(conn, user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Admin: list all active sessions across all users, joined with user email.
    pub async fn get_all_active_sessions(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<Vec<(Sessions, String)>, ServiceError> {
        self.session_repository
            .find_all_active(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Admin: revoke any session by ID without ownership check.
    /// Uses the macro-generated `update()` — no new repository method needed.
    pub async fn admin_revoke_session(
        &self,
        conn: &mut DbConnection<'_>,
        session_id: Uuid,
    ) -> Result<(), ServiceError> {
        self.session_repository
            .find_by_id(conn, session_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFoundError("Session not found".to_string()))?;

        let changeset = SessionRevocationChangeset {
            revoked_at: Some(Utc::now()),
            revoked_reason: Some(RevocationReason::UserLogout),
        };

        self.session_repository
            .update(conn, session_id, changeset)
            .await
            .map(|_| ())
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Admin: bulk-revoke sessions by their IDs.
    pub async fn admin_revoke_sessions_bulk(
        &self,
        conn: &mut DbConnection<'_>,
        ids: &[Uuid],
    ) -> Result<usize, ServiceError> {
        self.session_repository
            .revoke_sessions_by_ids(conn, ids)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn get_sessions_for_user(
        &self,
        conn: &mut DbConnection<'_>,
        user_id: Uuid,
    ) -> Result<Vec<Sessions>, ServiceError> {
        use crate::schema::sessions::{revoked_at as col_revoked_at, user_id as col_user_id};
        use diesel::{BoolExpressionMethods, ExpressionMethods};

        self.session_repository
            .find_where(conn, col_user_id.eq(user_id).and(col_revoked_at.is_null()))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn get_session_for_user(
        &self,
        conn: &mut DbConnection<'_>,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<Sessions, ServiceError> {
        let session = self
            .session_repository
            .find_by_id(conn, session_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFoundError("Session not found".to_string()))?;

        if session.user_id != user_id {
            return Err(ServiceError::NotFoundError("Session not found".to_string()));
        }
        Ok(session)
    }

    /// Override the access token TTL (primarily used in tests to generate expired tokens).
    pub fn with_access_ttl(mut self, ttl: Duration) -> Self {
        self.access_ttl = ttl;
        self
    }

    /// Issue a new PASETO v4.local access + refresh token pair for a user session.
    ///
    /// The caller is responsible for:
    /// 1. Creating the `Sessions` row in the DB (storing `refresh_token_hash`).
    /// 2. Returning both tokens to the client exactly once.
    ///
    /// Claims baked into the access token:
    /// - `ev` (email_verified) is captured at login time. Users must re-login after verifying
    ///   their email to receive a token with `ev: true`.
    /// - `mcp` (must_change_password) forces the user through the password-change gate.
    pub fn generate_session_token(
        &self,
        user: &User,
        session_id: &Uuid,
        roles: &Vec<Role>,
        identity: &UserIdentity,
    ) -> Result<SessionTokens, ServiceError> {
        // exp claim requires RFC3339 string and uses TryFrom per docs
        let exp_rfc3339 = (Utc::now() + self.access_ttl).to_rfc3339();

        let email_verified = identity.email_verified.unwrap_or(false);
        let must_change_password = identity.must_change_password;
        let usr = json!({
            "email": identity.email,
            "first_name": user.first_name,
            "last_name": user.last_name,
        });
        let claim_roles: Vec<&'static str> = roles.iter().map(|r| r.type_.as_str()).collect();

        // v4.local access token (encrypted)
        let sub = "Sentinel:Session".to_string();
        let access_token = PasetoBuilder::<V4, Local>::default()
            .set_claim(SubjectClaim::from(sub.as_str()))
            .set_claim(CustomClaim::try_from(("sid", session_id.to_string()))?)
            .set_claim(CustomClaim::try_from(("uid", user.user_id.to_string()))?)
            .set_claim(CustomClaim::try_from(("usr", usr))?) // JSON object claim
            .set_claim(CustomClaim::try_from(("roles", claim_roles))?) // JSON array claim
            .set_claim(CustomClaim::try_from(("ev", email_verified))?) // email verified claim
            .set_claim(CustomClaim::try_from(("mcp", must_change_password))?) // must change password
            // add issuer claim
            .set_claim(ExpirationClaim::try_from(exp_rfc3339.as_str())?)
            // seal key
            .build(&self.key)?;

        // opaque refresh token + hash (store hash only)
        let refresh_token = self.generate_refresh_token();
        let refresh_token_hash = sha256_hex(&refresh_token);

        Ok(SessionTokens {
            access_token,
            refresh_token,
            refresh_token_hash,
        })
    }

    /// Decrypt and validate a PASETO access token, returning its claims as an [`AuthContext`].
    ///
    /// This is a pure in-memory operation — no database call is made.
    /// The `exp` claim is validated by the PASETO library; expired tokens return
    /// [`ServiceError::ExpiredTokenError`].
    ///
    /// Note: A revoked session's token is still cryptographically valid. To enforce
    /// session revocation you would need to also look up `sessions.revoked_at`.
    pub fn authenticate_session_token(
        &self,
        auth_token: &str,
    ) -> Result<AuthContext, ServiceError> {
        let key = &self.key;

        // decrypt token
        let parsed_token = PasetoParser::<V4, Local>::default().parse(auth_token, key)?;

        // TODO: Add more complex checks here
        let user_id = parsed_token["uid"].as_str();
        let parsed_user_id = Uuid::parse_str(user_id.expect("Uid must be defined"))
            .map_err(|_| ServiceError::AuthenticationError("Invalid uid".to_string()))?;

        let session_id = parsed_token["sid"].as_str();
        let parsed_session_id = Uuid::parse_str(session_id.expect("Sid must be defined"))
            .map_err(|_| ServiceError::AuthenticationError("Invalid sid".to_string()))?;

        let usr = &parsed_token["usr"];
        let email = usr["email"]
            .as_str()
            .ok_or_else(|| ServiceError::AuthenticationError("Email must be defined".to_string()));
        let first_name = usr["first_name"].as_str().ok_or_else(|| {
            ServiceError::AuthenticationError("First name must be defined".to_string())
        });
        let last_name = usr["last_name"].as_str().ok_or_else(|| {
            ServiceError::AuthenticationError("Last name must be defined".to_string())
        });
        let roles: &Value = &parsed_token.clone()["roles"];
        let email_verified = parsed_token["ev"].as_bool().unwrap_or(false);
        let must_change_password = parsed_token["mcp"].as_bool().unwrap_or(false);
        let scope = parsed_token["scope"].as_str().map(String::from);
        let policy_test_id = parsed_token["pid"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok());

        let auth_context = AuthContext {
            user_id: parsed_user_id,
            session_id: parsed_session_id,
            email: email?.to_string(),
            first_name: first_name?.to_string(),
            last_name: last_name?.to_string(),
            roles: roles.clone(),
            email_verified,
            must_change_password,
            scope,
            policy_test_id,
        };
        tracing::debug!("Got auth context from token {:#?}", auth_context);
        Ok(auth_context)
    }

    /// Validate a raw refresh token against the database.
    ///
    /// 1. Hashes the raw token with SHA-256.
    /// 2. Looks up the `sessions` row by hash.
    /// 3. Rejects revoked or expired sessions.
    ///
    /// Returns the full `Sessions` row on success so the caller can read
    /// `user_id`, `session_id`, etc.
    pub async fn validate_refresh_token(
        &self,
        conn: &mut DbConnection<'_>,
        raw_token: &str,
    ) -> Result<Sessions, ServiceError> {
        use crate::schema::sessions::refresh_token_hash as col_rt_hash;
        use diesel::ExpressionMethods;

        let hash = sha256_hex(raw_token);
        let sessions = self
            .session_repository
            .find_where(conn, col_rt_hash.eq(hash))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let session = sessions.into_iter().next().ok_or_else(|| {
            ServiceError::InvalidTokenError("Refresh token not found".to_string())
        })?;

        if session.revoked_at.is_some() {
            return Err(ServiceError::InvalidTokenError(
                "Refresh token has been revoked".to_string(),
            ));
        }
        if session.refresh_token_expires_at < Utc::now() {
            return Err(ServiceError::ExpiredTokenError(
                "Refresh token has expired".to_string(),
            ));
        }
        Ok(session)
    }

    /// Rotate a session's refresh token: replace the stored hash, reset expiry, and
    /// update `last_used_at`. Called during the token-refresh flow after the old token
    /// has been validated and a new token pair has been generated.
    pub async fn rotate_session(
        &self,
        conn: &mut DbConnection<'_>,
        session_id: Uuid,
        new_hash: String,
    ) -> Result<(), ServiceError> {
        let changeset = SessionRotationChangeset {
            refresh_token_hash: new_hash,
            refresh_token_expires_at: Utc::now() + self.refresh_ttl,
            last_used_at: Some(Utc::now()),
        };
        self.session_repository
            .update(conn, session_id, changeset)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Issue a short-lived (15-min) PASETO token for the MFA challenge step.
    /// sub = "Sentinel:MfaChallenge" — no session row is created.
    pub fn generate_mfa_challenge_token(&self, user_id: Uuid) -> Result<String, ServiceError> {
        let exp = (Utc::now() + Duration::minutes(15)).to_rfc3339();
        PasetoBuilder::<V4, Local>::default()
            .set_claim(SubjectClaim::from("Sentinel:MfaChallenge"))
            .set_claim(CustomClaim::try_from(("uid", user_id.to_string()))?)
            .set_claim(ExpirationClaim::try_from(exp.as_str())?)
            .build(&self.key)
            .map_err(|e| ServiceError::TokenBuildError(e.to_string()))
    }

    /// Validate an MFA challenge token and return the embedded user_id.
    pub fn verify_mfa_challenge_token(&self, token: &str) -> Result<Uuid, ServiceError> {
        let parsed = PasetoParser::<V4, Local>::default().parse(token, &self.key)?;
        let sub = parsed["sub"].as_str().unwrap_or("");
        if sub != "Sentinel:MfaChallenge" {
            return Err(ServiceError::InvalidTokenError(
                "Not an MFA challenge token".to_string(),
            ));
        }
        let uid = parsed["uid"]
            .as_str()
            .ok_or_else(|| ServiceError::InvalidTokenError("Missing uid".to_string()))?;
        Uuid::parse_str(uid).map_err(|_| ServiceError::InvalidTokenError("Invalid uid".to_string()))
    }

    /// Issue a short-lived (5-min) PASETO policy test token.
    /// sub = "Sentinel:PolicyTest", scope = "policy_test" — no session row is created.
    /// The token carries the roles and policy_id to be tested but is NOT path-locked.
    pub fn generate_test_token(
        &self,
        roles: &[String],
        policy_id: Uuid,
        version: i64,
    ) -> Result<String, ServiceError> {
        let exp = (Utc::now() + Duration::minutes(5)).to_rfc3339();
        let synthetic_uid = Uuid::new_v4();
        let synthetic_sid = Uuid::new_v4();
        let usr = json!({
            "email": "probe@sentinel.internal",
            "first_name": "Sentinel",
            "last_name": "Probe",
        });
        let roles_ref: Vec<&str> = roles.iter().map(String::as_str).collect();
        PasetoBuilder::<V4, Local>::default()
            .set_claim(SubjectClaim::from("Sentinel:PolicyTest"))
            .set_claim(CustomClaim::try_from(("uid", synthetic_uid.to_string()))?)
            .set_claim(CustomClaim::try_from(("sid", synthetic_sid.to_string()))?)
            .set_claim(CustomClaim::try_from(("usr", usr))?)
            .set_claim(CustomClaim::try_from(("roles", roles_ref))?)
            .set_claim(CustomClaim::try_from(("ev", true))?)
            .set_claim(CustomClaim::try_from(("mcp", false))?)
            .set_claim(CustomClaim::try_from(("scope", "policy_test"))?)
            .set_claim(CustomClaim::try_from(("pid", policy_id.to_string()))?)
            .set_claim(CustomClaim::try_from(("pver", version))?)
            .set_claim(ExpirationClaim::try_from(exp.as_str())?)
            .build(&self.key)
            .map_err(|e| ServiceError::TokenBuildError(e.to_string()))
    }

    /// Generate a cryptographically random refresh token prefixed with `rt_`.
    /// Uses `OsRng` (not `ThreadRng`) to stay `Send` across `.await` points.
    fn generate_refresh_token(&self) -> String {
        let mut buf = vec![0u8; self.refresh_bytes];
        OsRng.fill_bytes(&mut buf);
        format!("rt_{}", URL_SAFE_NO_PAD.encode(buf))
    }
}

/// Compute the SHA-256 digest of a string and return it as a lowercase hex string.
/// Used to hash refresh tokens and API tokens before storing them in the database.
fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}
