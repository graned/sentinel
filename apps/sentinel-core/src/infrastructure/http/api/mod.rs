//! HTTP API surface — handlers, DTOs, middleware, routing, OpenAPI spec, and
//! shared response types.

pub mod dtos;
pub mod handlers;
pub mod middlewares;
pub mod openapi;
pub mod routes;
pub mod types;

pub use types::*;
