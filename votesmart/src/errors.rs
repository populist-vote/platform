#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Failed to parse environment variable.  Ensure you have a VOTESMART_API_KEY set in your .env file")]
    Var(#[from] std::env::VarError),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error("Failed to fetch from API")]
    Api,
}
