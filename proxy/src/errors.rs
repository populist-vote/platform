#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DbError(#[from] sqlx::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    VarError(#[from] std::env::VarError),

    #[error(transparent)]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to fetch from API")]
    ApiError,
}
