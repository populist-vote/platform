use async_graphql::ErrorExtensions;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    S3Error(#[from] anyhow::Error),

    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),

    #[error("BadInput (field: {field:?}, reason: {message:?})")]
    BadInput { field: String, message: String },

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

impl ErrorExtensions for Error {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_err, e| match self {
            Error::BadInput { field, message } => {
                e.set("code", "BAD_USER_INPUT");
                e.set("field", field.as_str());
                e.set("message", message.as_str());
            }
            _error => {
                e.set("code", "INTERNAL_SERVER_ERROR");
            }
        })
    }
}
