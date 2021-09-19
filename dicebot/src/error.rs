use std::net::AddrParseError;

use crate::commands::CommandError;
use crate::config::ConfigError;
use crate::db::errors::DataError;
use thiserror::Error;
use tonic::metadata::errors::InvalidMetadataValue;

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

    #[error("future canceled")]
    FutureCanceledError,

    //de = deserialization
    #[error("toml parsing error")]
    TomlParsingError(#[from] toml::de::Error),

    #[error("i/o error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("dice parsing error: {0}")]
    DiceParsingError(#[from] crate::parser::dice::DiceParsingError),

    #[error("command parsing error: {0}")]
    CommandParsingError(#[from] crate::commands::parser::CommandParsingError),

    #[error("dice rolling error: {0}")]
    DiceRollingError(#[from] DiceRollingError),

    #[error("variable parsing error: {0}")]
    VariableParsingError(#[from] crate::parser::variables::VariableParsingError),

    #[error("legacy parsing error")]
    NomParserError(nom::error::ErrorKind),

    #[error("legacy parsing error: not enough data")]
    NomParserIncomplete,

    #[error("variables not yet supported")]
    VariablesNotSupported,

    #[error("too many commands or message was too large")]
    MessageTooLarge,

    #[error("could not convert to proper integer type")]
    TryFromIntError(#[from] std::num::TryFromIntError),

    #[error("identifier error: {0}")]
    IdentifierError(#[from] matrix_sdk::ruma::identifiers::Error),

    #[error("password creation error: {0}")]
    PasswordCreationError(argon2::Error),

    #[error("account does not exist, or password incorrect")]
    AuthenticationError,

    #[error("user account does not exist, try registering")]
    AccountDoesNotExist,

    #[error("user account already exists")]
    AccountAlreadyExists,

    #[error("room name or id does not exist")]
    RoomDoesNotExist,

    #[error("tonic transport error: {0}")]
    TonicTransportError(#[from] tonic::transport::Error),

    #[error("address parsing error: {0}")]
    AddressParseError(#[from] AddrParseError),

    #[error("invalid metadata value: {0}")]
    TonicInvalidMetadata(#[from] InvalidMetadataValue),
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
