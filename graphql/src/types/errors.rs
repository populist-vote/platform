use async_graphql::ErrorExtensions;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    S3Error(#[from] anyhow::Error),

    #[error(transparent)]
    DatabaseError(#[from] db::Error),

    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error("BadInput (field: {field:?}, reason: {message:?})")]
    BadInput { field: String, message: String },

    #[error("Please check the format of the IDs you provided")]
    UuidError(#[from] uuid::Error),

    #[error(transparent)]
    VarError(#[from] std::env::VarError),

    #[error(transparent)]
    GeocodioError(#[from] geocodio::Error),

    #[error("A user already exists with this email")]
    UserExistsError,

    #[error("Your password was incorrect")]
    PasswordError,

    #[error("Your email or username was not found in our database")]
    EmailOrUsernameNotFound,

    #[error("No user was found in our database")]
    UserNotFound,

    #[error("This username is already taken")]
    UsernameTaken,

    #[error(transparent)]
    AuthError(#[from] auth::errors::Error),

    #[error("No user authentication token was provided with request")]
    Unauthorized,

    #[error("Your email address could not be confirmed")]
    ConfirmationError,

    #[error("We don't have an account associated with that email")]
    EmailNotFound,

    #[error("Passwords do not match")]
    PasswordsDoNotMatch,

    #[error("Your password is not strong enough")]
    PasswordEntropy,

    #[error("Reset token was invalid or expired")]
    ResetTokenInvalid,
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
