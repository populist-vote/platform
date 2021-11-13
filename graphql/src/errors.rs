#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    S3Error(#[from] anyhow::Error),

    #[error(transparent)]
    VarError(#[from] std::env::VarError),
}
