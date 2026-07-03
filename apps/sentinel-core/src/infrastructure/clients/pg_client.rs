use diesel::{ConnectionError, ConnectionResult};
use diesel_async::pooled_connection::bb8::{Pool, PooledConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::ManagerConfig;
use diesel_async::pooled_connection::PoolError;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use rustls::ClientConfig;
use rustls_platform_verifier::ConfigVerifierExt;
use std::time::Duration;

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConnection<'a> = PooledConnection<'a, AsyncPgConnection>;

#[derive(Clone, Debug)]
pub struct PostgresClient {
    pub pool: DbPool,
}

fn establish_connection(config: &str) -> BoxFuture<'_, ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        let rustls_config = ClientConfig::with_platform_verifier();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                tracing::error!("postgres connection error: {}", e);
            }
        });
        AsyncPgConnection::try_from(client).await
    };
    fut.boxed()
}

impl PostgresClient {
    pub async fn new(database_url: &str) -> Result<Self, bb8::RunError<PoolError>> {
        let mut config = ManagerConfig::default();
        config.custom_setup = Box::new(establish_connection);
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
            database_url,
            config,
        );
        let pool = Pool::builder()
            .max_size(20)
            .min_idle(Some(5))
            .connection_timeout(Duration::from_secs(30))
            .test_on_check_out(true)
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
