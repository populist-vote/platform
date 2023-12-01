use crate::Error;
use once_cell::sync::OnceCell;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub type DatabaseConnection = sqlx::pool::PoolConnection<Postgres>;
#[derive(Debug, Clone)]
pub struct DatabasePool {
    pub connection: Pool<Postgres>,
}

impl DatabasePool {
    pub async fn new() -> Result<DatabasePool, Error> {
        dotenv::dotenv().ok();
        let db_url = std::env::var("DATABASE_URL").expect("Could not parse DATABSE_URL");

        let pool = PgPoolOptions::new()
            .max_connections(16)
            .connect(&db_url)
            .await?;

        Ok(DatabasePool { connection: pool })
    }

    pub async fn acquire(&self) -> Result<DatabaseConnection, Error> {
        Ok(self.connection.acquire().await?)
    }
}

static POOL: OnceCell<DatabasePool> = OnceCell::new();

pub async fn init_pool() -> Result<(), Error> {
    POOL.set(DatabasePool::new().await?).unwrap();
    Ok(())
}

pub async fn pool<'a>() -> &'a DatabasePool {
    POOL.get().unwrap()
}
