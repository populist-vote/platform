pub use lazy_static::lazy_static;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::{mysql::MySqlPoolOptions, Connection};
use sqlx::{ConnectOptions, Error, Acquire};
use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;

pub type DatabaseConnection = sqlx::pool::PoolConnection<Database>;
pub type Database = sqlx::mysql::MySql;
pub type Transaction<'c> = sqlx::Transaction<'c, Database>;

#[derive(Clone, Debug)]
pub struct DatabasePool {
    inner_sqlx: Arc<sqlx::mysql::MySqlPool>,
    inner_read_only: Option<Arc<sqlx::mysql::MySqlPool>>,
}

pub struct DatabasePoolOptions {
    pub min_size: u32,
    pub max_size: u32,
    pub connect_timeout: Duration,
}

impl DatabasePool {
    pub async fn new_from_config(url: &str, read_only_url: &Option<String>) -> Result<Self, Error> {
        let db_options = crate::DatabasePoolOptions {
            min_size: *crate::MIN_DATABASE_POOL_SIZE,
            max_size: *crate::MAX_DATABASE_POOL_SIZE,
            connect_timeout: *crate::DATABASE_CONNECTION_TIMEOUT,
        };

        DatabasePool::new(&db_options, &url, read_only_url).await
    }

    pub async fn new(
        options: &DatabasePoolOptions,
        url: &str,
        read_only_url: &Option<String>,
    ) -> Result<Self, Error> {
        let enable_logging = enable_sqlx_logging();

        let mut connection_options: MySqlConnectOptions = url.parse()?;

        if !enable_logging {
            connection_options.disable_statement_logging();
        }

        let pool = MySqlPoolOptions::new()
            .connect_timeout(options.connect_timeout)
            .min_connections(options.min_size)
            .max_connections(options.max_size)
            .connect_with(connection_options)
            .await?;

        let read_only_pool = if let Some(url) = read_only_url {
            let mut connection_options: MySqlConnectOptions = url.parse()?;

            if !enable_logging {
                connection_options.disable_statement_logging();
            }

            Some(
                MySqlPoolOptions::new()
                    .connect_timeout(options.connect_timeout)
                    .max_connections(options.max_size)
                    .min_connections(options.min_size)
                    .connect_with(connection_options)
                    .await?,
            )
        } else {
            None
        };

        Ok(Self {
            inner_sqlx: Arc::new(pool),
            inner_read_only: read_only_pool.map(|p| Arc::new(p)),
        })
    }

    pub async fn acquire(&self) -> Result<DatabaseConnection, Error> {
        Ok(self.inner_sqlx.acquire().await?)
    }

    ///returns a read only connection if available. will use the primary writer otherwise
    pub async fn acquire_read_only(&self) -> Result<DatabaseConnection, Error> {
        if let Some(ro) = &self.inner_read_only {
            Ok(ro.acquire().await?)
        } else {
            self.acquire().await
        }
    }

    pub async fn begin_db_tx<'c>(
        &self,
        conn: &'c mut DatabaseConnection,
    ) -> Result<Transaction<'c>, Error> {
        conn.begin().await.map_err(|e| Error::from(e))
    }
}

pub fn enable_sqlx_logging() -> bool {
    if let Ok(value) = std::env::var("SQLX_LOG") {
        value == "1"
    } else {
        false
    }
}

lazy_static! {
    pub static ref DATABASE_CONNECTION_TIMEOUT: std::time::Duration = {
        let seconds = std::env::var("DATABASE_CONNECTION_TIMEOUT")
            .unwrap_or("5".into())
            .parse::<u64>()
            .unwrap_or(5);
        std::time::Duration::from_secs(seconds)
    };

    //always keep this many connections available in the pool
    pub static ref MIN_DATABASE_POOL_SIZE: u32 = std::env::var("MIN_DATABASE_POOL_SIZE")
        .unwrap_or("1".into())
        .parse::<u32>()
        .unwrap_or(1);

    pub static ref MAX_DATABASE_POOL_SIZE: u32 = std::env::var("MAX_DATABASE_POOL_SIZE")
        .unwrap_or("16".into())
        .parse::<u32>()
        .unwrap_or(16);
}
