//! `sentinel-core` — main Axum HTTP service for the Sentinel Auth platform.
//!
//! # Architecture
//!
//! The codebase follows Clean Architecture with a strict inward dependency rule:
//!
//! ```text
//! Infrastructure (HTTP handlers, DB clients, external clients)
//!     ↑
//! Application (use-case orchestrators: AuthApplication, MfaApplication, …)
//!     ↑
//! Services (single-responsibility business logic)
//!     ↑
//! Domain (entities from schema_models.rs, repository traits)
//! ```
//!
//! # Module layout
//!
//! | Module            | Role |
//! |-------------------|------|
//! | `applications`    | Multi-service flows & transaction boundaries |
//! | `services`        | Single-responsibility business logic |
//! | `domain`          | Repository impls backed by the `impl_repository!` macro |
//! | `infrastructure`  | HTTP layer (handlers, DTOs, middleware, routing) + DB client |
//! | `schema`          | Auto-generated Diesel table DSL (do not edit by hand) |
//! | `schema_models`   | Auto-generated Diesel entity structs |
//! | `schema_enums`    | Auto-generated Diesel enum types |
//! | `errors`          | Centralised error hierarchy: Domain → Repository → Service → Api |
//! | `utils`           | `impl_repository!` macro and entity mapping helpers |

pub mod applications;
pub mod domain;
pub mod errors;
pub mod infrastructure;
pub mod schema;
pub mod schema_enums;
pub mod schema_enums_impls;
pub mod schema_models;
pub mod services;
pub mod utils;

pub use applications::*;
pub use domain::*;
pub use errors::*;
pub use infrastructure::*;
pub use schema_enums::*;
pub use schema_models::*;
pub use services::*;
pub use utils::*;
