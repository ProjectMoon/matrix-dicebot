use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    /// Sync token couldn't be found.
    #[error("the sync token could not be retrieved")]
    SyncTokenRequired,

    #[error("the message should not be processed because it failed validation")]
    ShouldNotProcessError,

    #[error("no cache directory found")]
    NoCacheDirectoryError,

    #[error("could not parse URL")]
    UrlParseError(#[from] url::ParseError),

    #[error("uncategorized matrix SDK error")]
    MatrixError(#[from] matrix_sdk::Error),

    #[error("uncategorized matrix SDK base error")]
    MatrixBaseError(#[from] matrix_sdk::BaseError),

    #[error("future canceled")]
    FutureCanceledError,

    #[error("tokio task join error")]
    TokioTaskJoinError(#[from] tokio::task::JoinError),

    //de = deserialization
    #[error("toml parsing error")]
    TomlParsingError(#[from] toml::de::Error),

    #[error("i/o error")]
    IoError(#[from] std::io::Error),
}
