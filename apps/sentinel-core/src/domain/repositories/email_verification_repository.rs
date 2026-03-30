//! Repository for the `email_verifications` table.
//!
//! Stores `ev_*` verification tokens (only the SHA-256 hash) linked to a user identity.
//! Tokens expire after 24 hours; `verified_at` is set when consumed.
//! The macro provides all needed CRUD — no custom methods are required.

use crate::impl_repository;
use crate::EmailVerification;
use uuid::Uuid;

impl_repository!(
    EmailVerificationRepository for EmailVerification,
    crate::schema::email_verifications::table,
    crate::schema::email_verifications::verification_id,
    Uuid
);
