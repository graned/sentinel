//! Application layer — use-case orchestrators that coordinate multi-service flows,
//! enforce transaction boundaries, and apply cross-cutting concerns (role guards).
//!
//! Each `*Application` struct is constructed once in [`crate::http::app::build_app`]
//! and shared across requests via `Arc<AppState>`.  Handlers call application methods
//! directly; they never import services.

pub mod admin_application;
pub mod admin_session_application;
pub mod api_token_application;
pub mod auth_application;
pub mod email_template_application;
pub mod federation_application;
pub mod insights_application;
pub mod mfa_application;
pub mod oidc_application;
pub mod policy_application;
pub mod system_application;
pub mod user_application;
pub mod user_password_application;

pub use admin_application::AdminApplication;
pub use admin_session_application::AdminSessionApplication;
pub use api_token_application::*;
pub use auth_application::*;
pub use email_template_application::*;
pub use federation_application::*;
pub use insights_application::InsightsApplication;
pub use mfa_application::*;
pub use oidc_application::*;
pub use policy_application::*;
pub use system_application::*;
pub use user_application::*;
pub use user_password_application::*;
