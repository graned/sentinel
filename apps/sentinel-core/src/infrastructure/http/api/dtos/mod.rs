//! Data transfer objects (DTOs) — request bodies, response types, and the
//! shared `AuthenticatedUserContext` used by handlers and middleware.
//!
//! All DTOs exposed in Swagger derive `utoipa::ToSchema` (or `IntoParams` for
//! query-param structs).  Incoming request DTOs derive `validator::Validate` and
//! are extracted via the [`ValidatedJson`](crate::http::api::routes::api_validation::ValidatedJson)
//! extractor which runs validation before the handler is called.

pub mod admin_dtos;
pub mod admin_session_dtos;
pub mod api_token_dtos;
pub mod auth_dtos;
pub mod email_template_dtos;
pub mod federation_dtos;
pub mod insights_dtos;
pub mod mfa_dtos;
pub mod oidc_dtos;
pub mod password_dtos;
pub mod policy_dtos;
pub mod system_dtos;
pub mod user_dtos;

pub use admin_dtos::*;
pub use admin_session_dtos::*;
pub use api_token_dtos::*;
pub use auth_dtos::*;
pub use email_template_dtos::*;
pub use federation_dtos::*;
pub use insights_dtos::*;
pub use mfa_dtos::*;
pub use oidc_dtos::*;
pub use password_dtos::*;
pub use policy_dtos::*;
pub use system_dtos::*;
pub use user_dtos::*;
