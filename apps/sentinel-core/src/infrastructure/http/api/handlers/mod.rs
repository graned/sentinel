//! Axum handler functions, grouped by feature area.
//!
//! Each sub-module contains the handler functions for a related set of endpoints.
//! Handlers are thin — they extract request data, call the appropriate application
//! method, and return the result. Business logic lives in the application / service
//! layers, not here.

pub mod admin_oidc_handlers;
pub mod admin_role_handlers;
pub mod admin_session_handlers;
pub mod admin_user_handlers;
pub mod api_error_handlers;
pub mod api_token_handlers;
pub mod auth_handlers;
pub mod email_template_handlers;
pub mod federation_handlers;
pub mod mfa_handlers;
pub mod oidc_handlers;
pub mod password_handlers;
pub mod system_handlers;
pub mod user_handlers;

pub use admin_oidc_handlers::*;
pub use admin_role_handlers::*;
pub use admin_session_handlers::*;
pub use admin_user_handlers::*;
pub use api_error_handlers::*;
pub use api_token_handlers::*;
pub use auth_handlers::*;
pub use email_template_handlers::*;
pub use federation_handlers::*;
pub use mfa_handlers::*;
pub use oidc_handlers::*;
pub use password_handlers::*;
pub use system_handlers::*;
pub use user_handlers::*;
