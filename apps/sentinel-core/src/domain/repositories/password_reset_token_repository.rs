//! Repository for the `password_reset_tokens` table.
//!
//! Stores `pr_*` password-reset tokens (only the SHA-256 hash) linked to a user identity.
//! Tokens expire after 1 hour; `used_at` is set when consumed.
//! The macro provides all needed CRUD — custom lookup by hash is done in the service
//! layer via `find_where(col_token_hash.eq(hash))`.

use crate::impl_repository;
use crate::PasswordResetToken;
use uuid::Uuid;

impl_repository!(
    PasswordResetTokenRepository for PasswordResetToken,
    crate::schema::password_reset_tokens::table,
    crate::schema::password_reset_tokens::reset_token_id,
    Uuid
);
