#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    VarError(#[from] std::env::VarError),

    #[error("JWT Error: {0}")]
    JwtError(#[source] jsonwebtoken::errors::Error),

    #[error("You are not authorized to perform this action")]
    Unauthorized,
}
