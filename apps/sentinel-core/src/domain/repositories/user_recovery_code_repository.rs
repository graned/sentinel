//! Repository for the `user_recovery_codes` table.
//!
//! Recovery codes are one-time-use backup codes for MFA.  8 codes are generated and
//! stored as SHA-256 hashes when TOTP enrollment is confirmed.  Using a code sets
//! `used_at`; codes are never hard-deleted (audit trail).
//!
//! `delete_all_for_user` is used to regenerate the set — the old codes are wiped
//! before the new 8 hashes are inserted.

use crate::impl_repository;
use crate::UserRecoveryCode;
use uuid::Uuid;

impl_repository!(
    UserRecoveryCodeRepository for UserRecoveryCode,
    crate::schema::user_recovery_codes::table,
    crate::schema::user_recovery_codes::user_recovery_code_id,
    Uuid
);

impl UserRecoveryCodeRepository {
    /// Hard-delete all recovery codes for a user.  Called before re-generating codes
    /// so the new set replaces the old one atomically within the same transaction.
    pub async fn delete_all_for_user(
        &self,
        conn: &mut crate::DbConnection<'_>,
        target_user_id: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::user_recovery_codes::dsl::{user_id, user_recovery_codes};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        diesel::delete(user_recovery_codes.filter(user_id.eq(target_user_id)))
            .execute(conn)
            .await
    }
}
