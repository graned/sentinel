//! Infrastructure layer — HTTP server, request routing, middleware, DTOs, and
//! database client.
//!
//! This is the outermost layer in the Clean Architecture stack.  It depends on
//! the application layer but nothing below it depends on infrastructure.

pub mod clients;
pub mod http;

pub use clients::*;
pub use http::*;
