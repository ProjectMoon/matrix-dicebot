use crate::cofd::dice::DiceRollingError;
use crate::commands::CommandError;
use crate::config::ConfigError;
use crate::db::errors::DataError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("configuration error: {0}")]
    ConfigurationError(#[from] ConfigError),

    /// Sync token couldn't be found.
    #[error("the sync token could not be retrieved")]
    SyncTokenRequired,

    #[error("command error: {0}")]
    CommandError(#[from] CommandError),

    #[error("database error: {0}")]
    DataError(#[from] DataError),

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

    //de = deserialization
    #[error("toml parsing error")]
    TomlParsingError(#[from] toml::de::Error),

    #[error("i/o error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("dice parsing error: {0}")]
    DiceParsingError(#[from] crate::parser::DiceParsingError),

    #[error("command parsing error: {0}")]
    CommandParsingError(#[from] crate::commands::parser::CommandParsingError),

    #[error("dice pool roll error: {0}")]
    DiceRollingError(#[from] DiceRollingError),

    #[error("variable parsing error: {0}")]
    VariableParsingError(#[from] crate::variables::VariableParsingError),

    #[error("legacy parsing error")]
    NomParserError(nom::error::ErrorKind),

    #[error("legacy parsing error: not enough data")]
    NomParserIncomplete,

    #[error("variables not yet supported")]
    VariablesNotSupported,

    #[error("database error")]
    DatabaseErrror(#[from] sled::Error),
}
