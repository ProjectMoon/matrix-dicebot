use sled::transaction::{TransactionError, UnabortableTransactionError};
use thiserror::Error;

//TODO better combining of key and value in certain errors (namely
//I32SchemaViolation).
#[derive(Error, Debug)]
pub enum DataError {
    #[error("value does not exist for key: {0}")]
    KeyDoesNotExist(String),

    #[error("expected i32, but i32 schema was violated")]
    I32SchemaViolation,

    #[error("expected string, but utf8 schema was violated: {0}")]
    Utf8chemaViolation(#[from] std::str::Utf8Error),

    #[error("internal database error: {0}")]
    InternalError(#[from] sled::Error),

    #[error("transaction error: {0}")]
    TransactionError(#[from] sled::transaction::TransactionError),

    #[error("unabortable transaction error: {0}")]
    UnabortableTransactionError(#[from] UnabortableTransactionError),
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
