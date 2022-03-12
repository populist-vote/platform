#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),

    #[error("Your email or username was not found in our database")]
    EmailOrUsernameNotFound,

    #[error("Reset token has expired or is not valid")]
    ResetTokenInvalid,
}
