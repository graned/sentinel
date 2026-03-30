//! Repository for the `api_tokens` table.
//!
//! API tokens are long-lived opaque credentials (`sat_<hex>`) for programmatic access.
//! Only the SHA-256 hash of the raw token is stored — the plaintext is returned exactly
//! once at creation and never persisted.  Revocation is always a soft-delete (sets
//! `revoked_at`; no hard deletes).
//!
//! Custom methods beyond the macro-generated CRUD:
//! - [`find_by_token_hash`](ApiTokenRepository::find_by_token_hash) — lookup during
//!   authentication (hashes the presented token, queries by hash)
//! - [`revoke_all_for_user`](ApiTokenRepository::revoke_all_for_user) — bulk
//!   soft-revoke used by the admin "revoke all" action

use crate::impl_repository;
use crate::ApiToken;
use uuid::Uuid;

impl_repository!(
    ApiTokenRepository for ApiToken,
    crate::schema::api_tokens::table,
    crate::schema::api_tokens::api_token_id,
    Uuid
);

impl ApiTokenRepository {
    /// Find a token by its SHA-256 hash (used during validation).
    pub async fn find_by_token_hash<'a>(
        &self,
        conn: &mut crate::DbConnection<'a>,
        hash: &'a str,
    ) -> Result<Option<ApiToken>, crate::RepositoryError> {
        use crate::schema::api_tokens::token_hash as col_token_hash;
        use diesel::ExpressionMethods;

        let results = self
            .find_where(conn, col_token_hash.eq(hash))
            .await?;

        Ok(results.into_iter().next())
    }

    /// Bulk soft-revoke all tokens for a user (sets revoked_at = now()).
    pub async fn revoke_all_for_user(
        &self,
        conn: &mut crate::DbConnection<'_>,
        target_user_id: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::api_tokens::dsl::{api_tokens, revoked_at, user_id};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        diesel::update(
            api_tokens
                .filter(user_id.eq(target_user_id))
                .filter(revoked_at.is_null()),
        )
        .set(revoked_at.eq(chrono::Utc::now()))
        .execute(conn)
        .await
    }
}
