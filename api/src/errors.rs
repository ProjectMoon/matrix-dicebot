use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("user account does not exist: {0}")]
    UserDoesNotExist(String),

    #[error("invalid password for user account: {0}")]
    AuthenticationDenied(String),

    #[error("authentication token missing from request")]
    AuthenticationRequired,

    #[error("error decoding token: {0}")]
    TokenDecodingError(#[from] jsonwebtoken::errors::Error),
}
