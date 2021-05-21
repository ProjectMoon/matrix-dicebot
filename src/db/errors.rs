use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("value does not exist for key: {0}")]
    KeyDoesNotExist(String),

    #[error("too many entries")]
    TooManyEntries,

    #[error("expected i32, but i32 schema was violated")]
    I32SchemaViolation,

    #[error("unexpected or corruptd data bytes")]
    InvalidValue,

    #[error("expected string ref, but utf8 schema was violated: {0}")]
    Utf8RefSchemaViolation(#[from] std::str::Utf8Error),

    #[error("expected string, but utf8 schema was violated: {0}")]
    Utf8SchemaViolation(#[from] std::string::FromUtf8Error),

    #[error("data migration error: {0}")]
    MigrationError(#[from] crate::db::sqlite::migrator::MigrationError),

    #[error("internal database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("numeric conversion error")]
    NumericConversionError(#[from] TryFromIntError),
}
