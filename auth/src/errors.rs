#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    VarError(#[from] std::env::VarError),

    #[error(transparent)]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("You are not authorized to perform this action")]
    Unauthorized,
}
