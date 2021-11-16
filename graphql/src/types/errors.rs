#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    S3Error(#[from] anyhow::Error),

    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),

    #[error("Please check the format of the IDs you provided")]
    UuidError(#[from] uuid::Error),

    #[error(transparent)]
    VarError(#[from] std::env::VarError),

    #[error("Your password was incorrect")]
    PasswordError,

    #[error("Your email or username was not found in our database")]
    EmailOrUsernameNotFound(#[from] db::Error),

    #[error(transparent)]
    AuthError(#[from] auth::errors::Error),
}
