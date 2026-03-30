//! Route builders for the two top-level routers:
//! - `api_router` — `/v1/api/*` with the full middleware stack and Sentinel envelope
//! - `oauth_router` — `/oauth/*` and `/.well-known/*` with raw spec-compliant responses

pub mod api_router;
pub mod api_validation;
pub mod oauth_router;

// Re-export routes
pub use api_router::build_router;
