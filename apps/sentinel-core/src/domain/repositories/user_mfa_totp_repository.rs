//! Repository for the `user_mfa_totp` table.
//!
//! Stores the XChaCha20-Poly1305-encrypted TOTP secret and an `enabled` flag per user.
//! A row may exist with `enabled = false` during enrollment (after `totp_start` but before
//! `totp_confirm`); only rows with `enabled = true` represent active MFA.

use crate::impl_repository;
use crate::UserMfaTotp;
use uuid::Uuid;

impl_repository!(
    UserMfaTotpRepository for UserMfaTotp,
    crate::schema::user_mfa_totp::table,
    crate::schema::user_mfa_totp::user_mfa_totp_id,
    Uuid
);

impl UserMfaTotpRepository {
    /// Count users who have TOTP MFA actively enabled.
    pub async fn count_enabled(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<i64, diesel::result::Error> {
        use crate::schema::user_mfa_totp::dsl::{enabled, user_mfa_totp};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        user_mfa_totp
            .filter(enabled.eq(true))
            .count()
            .get_result(conn)
            .await
    }
}
