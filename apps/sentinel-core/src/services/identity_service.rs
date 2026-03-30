//! Identity service — manages `user_identities` rows (email + password hash).
//!
//! In Sentinel a **user** record (`users` table) holds account-level state while a
//! **user identity** record (`user_identities` table) holds the email address and
//! password hash for local (email+password) authentication.  This separation allows
//! multiple auth methods to be added per user in the future without schema changes.
//!
//! # Password hashing
//!
//! Passwords are **never** hashed in application code.  The `password_hash` column
//! stores the output of PostgreSQL's `crypt()` function (`pgcrypt` extension).
//! The [`fetch_identity_with_credentials`](crate::IdentitiesRepository::fetch_identity_with_credentials)
//! repository method passes the plaintext password to `crypt()` inside the SQL query;
//! if the hashes match Postgres returns the row, otherwise it returns nothing.
//!
//! # Email verification
//!
//! `email_verified` in `user_identities` is the source of truth.  It is set to `true`
//! by [`mark_email_verified`](IdentityService::mark_email_verified) after the
//! verification link is clicked.  The `ev` claim baked into PASETO tokens at login
//! time reflects the value **at login** — users must re-login after verifying.

use crate::schema::user_identities::{email, user_id};
use crate::{DbConnection, IdentitiesRepository, ServiceError, UserIdentity};
use diesel::ExpressionMethods;

use std::sync::Arc;

/// Provides identity (email + password) CRUD operations to the application layer.
pub struct IdentityService {
    identity_repository: Arc<IdentitiesRepository>,
}

impl IdentityService {
    pub fn new(identity_repository: Arc<IdentitiesRepository>) -> Self {
        Self {
            identity_repository,
        }
    }

    /// Returns `Ok(true)` if no identity with `email_to_verify` exists yet.
    /// Returns `Err(ValidationError)` if the email is already registered (prevents duplicates).
    pub async fn verify_email_availability(
        &self,
        conn: &mut DbConnection<'_>,
        email_to_verify: &str,
    ) -> Result<bool, ServiceError> {
        let email_value = email_to_verify.to_string(); // owned
                                                       // Verify that email is available
        let existing_identity = self
            .identity_repository
            .find_where(conn, email.eq(email_value))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !existing_identity.is_empty() {
            return Err(ServiceError::ValidationError(format!(
                "User with email {} already exists",
                email_to_verify
            )));
        }
        Ok(true)
    }

    /// Validate credentials by delegating to `fetch_identity_with_credentials`, which
    /// runs `crypt(password, password_hash)` inside the DB.
    /// Returns `Err(AuthenticationError)` for both "unknown email" and "wrong password"
    /// to prevent user enumeration via timing differences.
    pub async fn verify_identity_exists(
        &self,
        conn: &mut DbConnection<'_>,
        identity_email: &str,
        password: &str,
    ) -> Result<UserIdentity, ServiceError> {
        // Verify user identity exists
        let found_identity = self
            .identity_repository
            .fetch_identity_with_credentials(conn, identity_email, password)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if found_identity.is_none() {
            return Err(ServiceError::AuthenticationError(
                "Invalid credentials".to_string(),
            ));
        }

        Ok(found_identity.unwrap())
    }
    /// Return the first (primary) identity for a user, or `None` if none exists.
    /// In the current schema each user has exactly one identity, so this is effectively
    /// "get the email + status row for this user".
    pub async fn find_primary_identity_by_user_id(
        &self,
        conn: &mut DbConnection<'_>,
        target_user_id: uuid::Uuid,
    ) -> Result<Option<UserIdentity>, ServiceError> {
        let identities = self
            .identity_repository
            .find_where(conn, user_id.eq(target_user_id))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        Ok(identities.into_iter().next())
    }

    pub async fn create_identity(
        &self,
        conn: &mut DbConnection<'_>,
        identity: &UserIdentity,
    ) -> Result<UserIdentity, ServiceError> {
        self.identity_repository
            .create(conn, identity)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Set `email_verified = true` and update `verified_at` on the identity row.
    /// Called after the user clicks the verification link and the token is validated.
    pub async fn mark_email_verified(
        &self,
        conn: &mut DbConnection<'_>,
        target_identity_id: uuid::Uuid,
    ) -> Result<(), ServiceError> {
        #[derive(diesel::AsChangeset)]
        #[diesel(table_name = crate::schema::user_identities)]
        struct EmailVerifiedChangeset {
            email_verified: Option<bool>,
        }

        self.identity_repository
            .update(
                conn,
                target_identity_id,
                EmailVerifiedChangeset {
                    email_verified: Some(true),
                },
            )
            .await
            .map(|_| ())
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn find_identity_by_id(
        &self,
        conn: &mut DbConnection<'_>,
        identity_id: uuid::Uuid,
    ) -> Result<Option<UserIdentity>, ServiceError> {
        self.identity_repository
            .find_by_id(conn, identity_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Update the password hash for an identity and clear the `must_change_password` flag.
    ///
    /// `new_password` must already be the PostgreSQL `crypt(plaintext, gen_salt('bf'))` output
    /// — the application layer is responsible for hashing before calling this method.
    /// Also stamps `password_changed_at = now()`.
    pub async fn update_password(
        &self,
        conn: &mut DbConnection<'_>,
        identity_id: uuid::Uuid,
        new_password: String,
    ) -> Result<(), ServiceError> {
        #[derive(diesel::AsChangeset)]
        #[diesel(table_name = crate::schema::user_identities)]
        struct PasswordChangeset {
            password_hash: Option<String>,
            password_changed_at: Option<chrono::DateTime<chrono::Utc>>,
            must_change_password: bool,
        }

        self.identity_repository
            .update(
                conn,
                identity_id,
                PasswordChangeset {
                    password_hash: Some(new_password),
                    password_changed_at: Some(chrono::Utc::now()),
                    must_change_password: false,
                },
            )
            .await
            .map(|_| ())
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn find_identity_by_email<'a>(
        &self,
        conn: &mut DbConnection<'a>,
        target_email: &'a str,
    ) -> Result<Option<UserIdentity>, ServiceError> {
        let results = self
            .identity_repository
            .find_where(conn, email.eq(target_email))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        Ok(results.into_iter().next())
    }
}
