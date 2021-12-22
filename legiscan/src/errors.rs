#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Var(#[from] std::env::VarError),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error("Failed to fetch from API")]
    Api,
}
