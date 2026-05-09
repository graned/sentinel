//! User service — CRUD wrapper around [`UserRepository`].
//!
//! Thin adapter that converts `RepositoryError` → `ServiceError::DatabaseError` so
//! the application layer works with a single error type.  Business-level invariants
//! (e.g. email uniqueness) live in [`IdentityService`](crate::IdentityService) because
//! email addresses belong to `user_identities`, not `users`.
//!
//! Password hashing is delegated to PostgreSQL's `pgcrypt` extension — this service
//! never sees or stores plaintext passwords.

use crate::{DbConnection, ServiceError, User, UserRepository, UserStatus};

use std::sync::Arc;
use uuid::Uuid;

/// Provides user-account CRUD operations to the application layer.
pub struct UserService {
    user_repository: Arc<UserRepository>,
}

impl UserService {
    pub fn new(user_repository: Arc<UserRepository>) -> Self {
        Self { user_repository }
    }

    /// Insert a new `users` row.
    pub async fn create_user(
        &self,
        conn: &mut DbConnection<'_>,
        user: &User,
    ) -> Result<User, ServiceError> {
        self.user_repository
            .create(conn, user)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Fetch a user by primary key; returns `None` when not found.
    pub async fn find_user_by_id(
        &self,
        conn: &mut DbConnection<'_>,
        user_id: Uuid,
    ) -> Result<Option<User>, ServiceError> {
        self.user_repository
            .find_by_id(conn, user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn list_all_users(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<Vec<User>, ServiceError> {
        self.user_repository
            .list_all(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn paginate_users(
        &self,
        conn: &mut DbConnection<'_>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<User>, i64), ServiceError> {
        self.user_repository
            .paginate_all(conn, page, page_size)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn update_user_status(
        &self,
        conn: &mut DbConnection<'_>,
        user_id: Uuid,
        status: UserStatus,
    ) -> Result<User, ServiceError> {
        self.user_repository
            .update_status(conn, user_id, status)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Toggle the `mfa_required` flag — called by the admin when mandating MFA for a user.
    /// When set to `true`, the user sees a "Setup MFA" prompt on next login until they enroll.
    pub async fn set_mfa_required(
        &self,
        conn: &mut DbConnection<'_>,
        user_id: Uuid,
        required: bool,
    ) -> Result<User, ServiceError> {
        self.user_repository
            .update_mfa_required(conn, user_id, required)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn update_user_profile(
        &self,
        conn: &mut DbConnection<'_>,
        user_id: Uuid,
        first_name: Option<String>,
        last_name: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<Option<User>, ServiceError> {
        self.user_repository
            .update_user_profile(conn, user_id, first_name, last_name, avatar_url)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn delete_user(
        &self,
        conn: &mut DbConnection<'_>,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        self.user_repository
            .delete(conn, user_id)
            .await
            .map(|_| ())
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }
}
