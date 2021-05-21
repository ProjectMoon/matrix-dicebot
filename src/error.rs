use crate::config::ConfigError;
use crate::db::errors::DataError;
use crate::{commands::CommandError, db::sqlite::migrator};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("configuration error: {0}")]
    ConfigurationError(#[from] ConfigError),

    /// Sync token couldn't be found.
    #[error("the sync token could not be retrieved")]
    SyncTokenRequired,

    #[error("could not retrieve device id")]
    NoDeviceIdFound,

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

    #[error("error in matrix state store: {0}")]
    MatrixStateStoreError(#[from] matrix_sdk::StoreError),

    #[error("uncategorized matrix SDK error: {0}")]
    MatrixError(#[from] matrix_sdk::Error),

    #[error("uncategorized matrix SDK base error: {0}")]
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

    #[error("dice rolling error: {0}")]
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
    DatabaseError(#[from] sled::Error),

    #[error("database migration error: {0}")]
    SqliteError(#[from] migrator::MigrationError),

    #[error("too many commands or message was too large")]
    MessageTooLarge,

    #[error("could not convert to proper integer type")]
    TryFromIntError(#[from] std::num::TryFromIntError),
}

#[derive(Error, Debug)]
pub enum DiceRollingError {
    #[error("variable not found: {0}")]
    VariableNotFound(String),

    #[error("invalid amount")]
    InvalidAmount,

    #[error("dice pool expression too large")]
    ExpressionTooLarge,
}
