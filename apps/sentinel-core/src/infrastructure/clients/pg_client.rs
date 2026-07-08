use diesel::{ConnectionError, ConnectionResult};
use diesel_async::pooled_connection::bb8::{Pool, PooledConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::ManagerConfig;
use diesel_async::pooled_connection::PoolError;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use native_tls::TlsConnector;
use std::error::Error;
use std::time::Duration;

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConnection<'a> = PooledConnection<'a, AsyncPgConnection>;

#[derive(Clone, Debug)]
pub struct PostgresClient {
    pub pool: DbPool,
}

fn establish_connection(config: &str) -> BoxFuture<'_, ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        let mut builder = TlsConnector::builder();

        let cert_path = std::env::var("SSL_CERT_FILE")
            .ok()
            .filter(|p| !p.is_empty());

        let default_path = "/ca-certificate.crt";
        let resolved_path = cert_path
            .as_deref()
            .filter(|p| std::path::Path::new(p).exists())
            .or_else(|| {
                if std::path::Path::new(default_path).exists() {
                    Some(default_path)
                } else {
                    None
                }
            });

        if let Some(path) = resolved_path {
            let cert = std::fs::read(path).map_err(|e| {
                ConnectionError::BadConnection(format!("Failed to read CA cert {}: {}", path, e))
            })?;
            let cert = native_tls::Certificate::from_pem(&cert)
                .map_err(|e| ConnectionError::BadConnection(format!("Invalid CA cert: {}", e)))?;
            builder.add_root_certificate(cert);
            tracing::info!("Loaded CA certificate from {}", path);
        }

        let connector = builder
            .build()
            .map_err(|e| ConnectionError::BadConnection(format!("TLS builder error: {}", e)))?;
        let tls = postgres_native_tls::MakeTlsConnector::new(connector);

        let (client, conn) = tokio_postgres::connect(config, tls).await.map_err(|e| {
            tracing::error!("Connection failed: {}", e);
            if let Some(inner) = e.into_source() {
                tracing::error!("  caused by: {}", inner);
                if let Some(source) = inner.source() {
                    tracing::error!("    caused by: {}", source);
                    let mut s = source.source();
                    while let Some(cause) = s {
                        tracing::error!("      caused by: {}", cause);
                        s = cause.source();
                    }
                }
            }
            ConnectionError::BadConnection("TLS handshake failed — see logs above".to_string())
        })?;

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
