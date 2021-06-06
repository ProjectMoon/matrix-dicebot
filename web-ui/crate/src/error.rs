use graphql_client_web::ClientError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("api client error: {0}")]
    ApiClientError(#[from] ClientError),

    /// General API error, collecting errors from graphql server.
    #[error("error: {0}")]
    ApiError(String),
}
