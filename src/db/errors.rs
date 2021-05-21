use std::num::TryFromIntError;

use sled::transaction::{TransactionError, UnabortableTransactionError};
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

    #[error("internal database error: {0}")]
    InternalError(#[from] sled::Error),

    #[error("transaction error: {0}")]
    TransactionError(#[from] sled::transaction::TransactionError),

    #[error("unabortable transaction error: {0}")]
    UnabortableTransactionError(#[from] UnabortableTransactionError),

    #[error("data migration error: {0}")]
    MigrationError(#[from] crate::db::sqlite::migrator::MigrationError),

    #[error("deserialization error: {0}")]
    DeserializationError(#[from] bincode::Error),

    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("numeric conversion error")]
    NumericConversionError(#[from] TryFromIntError),
}

/// This From implementation is necessary to deal with the recursive
/// error type in the error enum. We defined a transaction error, but
/// the only place we use it is when converting from
/// sled::transaction::TransactionError<DataError>. This converter
/// extracts the inner data error from transaction aborted errors, and
/// forwards anything else onward as-is, but wrapped in DataError.
impl From<TransactionError<DataError>> for DataError {
    fn from(error: TransactionError<DataError>) -> Self {
        match error {
            TransactionError::Abort(data_err) => data_err,
            TransactionError::Storage(storage_err) => {
                DataError::TransactionError(TransactionError::Storage(storage_err))
            }
        }
    }
}

/// Automatically aborts transactions that hit a DataError by using
/// the try (question mark) operator when this trait implementation is
/// in scope.
impl From<DataError> for sled::transaction::ConflictableTransactionError<DataError> {
    fn from(error: DataError) -> Self {
        sled::transaction::ConflictableTransactionError::Abort(error)
    }
}
