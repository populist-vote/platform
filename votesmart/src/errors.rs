#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse environment variable.  Ensure you have a VOTESMART_API_KEY set in your .env file")]
    VarError(#[from] std::env::VarError),

    #[error(transparent)]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to fetch from API")]
    ApiError,
}
