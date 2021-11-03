pub mod models;
mod planetscale;

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Id(pub uuid::Uuid);
pub type DateTime = chrono::DateTime<chrono::Utc>;

pub use planetscale::{
    Database, DatabaseConnection, DatabasePool, DatabasePoolOptions, Transaction,
    DATABASE_CONNECTION_TIMEOUT, MAX_DATABASE_POOL_SIZE, MIN_DATABASE_POOL_SIZE,
};

pub struct Context {
    pub pool: DatabasePool,
}

impl Context {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}
