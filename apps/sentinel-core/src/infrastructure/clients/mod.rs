//! Infrastructure clients — database connection pool and any future external clients.

pub mod pg_client;

pub use pg_client::*;
