//! Domain repository implementations for all Sentinel database entities.
//!
//! Each module calls the [`impl_repository!`](crate::impl_repository) macro, which
//! generates 13 standard CRUD + pagination methods:
//!
//! | Method | Description |
//! |--------|-------------|
//! | `create` | Insert one row, return the persisted entity |
//! | `find_by_id` | Fetch by primary key → `Option<T>` |
//! | `find_where` | Fetch all rows matching a Diesel predicate |
//! | `update` | Apply a changeset to a single row by PK |
//! | `update_where` | Apply a changeset to a **single** row matching a predicate (`get_result`) |
//! | `delete` | Hard-delete by PK |
//! | `count_all` | Total row count |
//! | `paginate_all` | Offset-limit page → `(Vec<T>, total_count)` |
//! | … | (plus additional helpers) |
//!
//! > **`update_where` is single-row only.** It uses `.get_result()` internally, so
//! > passing a predicate that matches multiple rows will return only one and discard
//! > the rest. For bulk updates (e.g. revoking all sessions for a user) add a custom
//! > method to the repository's separate `impl` block that calls `.execute()` instead.
//!
//! Custom DB logic (joins, raw SQL aggregates, bulk mutations) is added in a separate
//! `impl RepositoryName { … }` block after the macro invocation, with no additional
//! imports at the top of the file — the macro already imports `DbConnection` internally,
//! and a duplicate `use crate::DbConnection;` at the top would cause an `E0252` error.

pub mod api_token_repository;
pub mod email_template_repository;
pub mod email_verification_repository;
pub mod password_reset_token_repository;
pub mod identities_repository;
pub mod oidc_auth_code_repository;
pub mod oidc_client_repository;
pub mod oidc_signing_key_repository;
pub mod policies_repository;
pub mod policy_versions_repository;
pub mod provider_configuration_reposiory;
pub mod role_repository;
pub mod session_repository;
pub mod user_mfa_totp_repository;
pub mod user_recovery_code_repository;
pub mod user_repository;
pub mod user_role_repository;

pub use api_token_repository::*;
pub use email_template_repository::*;
pub use email_verification_repository::*;
pub use password_reset_token_repository::*;
pub use identities_repository::*;
pub use oidc_auth_code_repository::*;
pub use oidc_client_repository::*;
pub use oidc_signing_key_repository::*;
pub use policies_repository::*;
pub use policy_versions_repository::*;
pub use provider_configuration_reposiory::*;
pub use role_repository::*;
pub use session_repository::*;
pub use user_mfa_totp_repository::*;
pub use user_recovery_code_repository::*;
pub use user_repository::*;
pub use user_role_repository::*;
