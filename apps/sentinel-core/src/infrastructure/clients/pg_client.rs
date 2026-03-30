//! PostgreSQL connection pool client backed by `diesel-async` + `bb8`.
//!
//! All application and repository code receives a `&mut DbConnection<'_>` (a
//! checked-out pooled connection) — they never hold the pool directly.
//!
//! # Connection pool settings
//!
//! | Setting | Value |
//! |---------|-------|
//! | max pool size | 20 |
//! | min idle connections | 5 |
//! | checkout timeout | 30 s |
//! | validate on checkout | yes |

use diesel_async::pooled_connection::bb8::{Pool, PooledConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::PoolError;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use std::time::Duration;

/// Shared `bb8` connection pool type alias.
pub type DbPool = Pool<AsyncPgConnection>;
/// A single checked-out connection borrowed from the pool for the duration of a request.
pub type DbConnection<'a> = PooledConnection<'a, AsyncPgConnection>;

/// Thin wrapper around a `diesel-async` `bb8` connection pool.
///
/// Clone-safe (the inner `Pool` is `Arc`-backed). Pass `Arc<PostgresClient>` to
/// application structs so they can acquire connections on demand.
#[derive(Clone, Debug)]
pub struct PostgresClient {
    pub pool: DbPool,
}

impl PostgresClient {
    /// Create a new pool connected to `database_url`.
    /// Returns an error if the initial connection test fails.
    pub async fn new(database_url: &str) -> Result<Self, bb8::RunError<PoolError>> {
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);

        let pool = Pool::builder()
            .max_size(20) // Maximum number of connections in pool
            .min_idle(Some(5)) // Minimum idle connections to maintain
            .connection_timeout(Duration::from_secs(30))
            .test_on_check_out(true) // Validate connection on checkout
            .build(manager)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub async fn get_conn(&self) -> Result<DbConnection<'_>, bb8::RunError<PoolError>> {
        self.pool.get().await
    }

    pub async fn health_check(&self) -> anyhow::Result<String> {
        let mut conn = self.get_conn().await?;
        diesel::sql_query("SELECT 1").execute(&mut conn).await?;
        Ok("Database is healthy".to_string())
    }
}
