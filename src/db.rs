use byteorder::LittleEndian;
use sled::transaction::abort;
use sled::transaction::{TransactionError, TransactionalTree, UnabortableTransactionError};
use sled::{Db, Tree};
use std::collections::HashMap;
use thiserror::Error;
use zerocopy::byteorder::I32;
use zerocopy::{AsBytes, LayoutVerified};

/// User variables are stored as little-endian 32-bit integers in the
/// database. This type alias makes the database code more pleasant to
/// read.
type LittleEndianI32Layout<'a> = LayoutVerified<&'a [u8], I32<LittleEndian>>;

const VARIABLE_COUNT_KEY: &'static str = "variable_count";

#[derive(Clone)]
pub struct Database {
    db: Db,
    variables: Tree,
    rooms: Tree,
}

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

fn to_key(room_id: &str, username: &str, variable_name: &str) -> Vec<u8> {
    let mut key = vec![];
    key.extend_from_slice(room_id.as_bytes());
    key.extend_from_slice(username.as_bytes());
    key.extend_from_slice(variable_name.as_bytes());
    key
}

fn room_variable_count_key(room_id: &str, username: &str) -> Vec<u8> {
    to_key(room_id, username, VARIABLE_COUNT_KEY)
}

fn to_prefix(room_id: &str, username: &str) -> Vec<u8> {
    let mut prefix = vec![];
    prefix.extend_from_slice(room_id.as_bytes());
    prefix.extend_from_slice(username.as_bytes());
    prefix
}

/// Convert bytes to an i32 with zero-copy deserialization. An error
/// is returned if the bytes do not represent an i32.
fn convert_i32(raw_value: &[u8]) -> Result<i32, DataError> {
    let layout = LittleEndianI32Layout::new_unaligned(raw_value.as_ref());

    if let Some(layout) = layout {
        let value: I32<LittleEndian> = *layout;
        Ok(value.get())
    } else {
        Err(DataError::I32SchemaViolation)
    }
}

/// Atomically alter the count of variables in the database, by the
/// given amount. Count cannot go below 0.
fn alter_room_variable_count(
    variables: &TransactionalTree,
    room_id: &str,
    username: &str,
    amount: i32,
) -> Result<i32, DataError> {
    let key = room_variable_count_key(room_id, username);
    let mut new_count = match variables.get(&key)? {
        Some(bytes) => convert_i32(&bytes)? + amount,
        None => amount,
    };

    if new_count < 0 {
        new_count = 0;
    }

    let db_value: I32<LittleEndian> = I32::new(new_count);
    variables.insert(key, db_value.as_bytes())?;
    Ok(new_count)
}

impl Database {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Database, DataError> {
        let db = sled::open(path)?;
        let variables = db.open_tree("variables")?;
        let rooms = db.open_tree("rooms")?;

        Ok(Database {
            db: db.clone(),
            variables: variables,
            rooms: rooms,
        })
    }

    pub async fn get_user_variables(
        &self,
        room_id: &str,
        username: &str,
    ) -> Result<HashMap<String, i32>, DataError> {
        let prefix = to_prefix(&room_id, &username);
        let prefix_len: usize = prefix.len();

        let variables: Result<Vec<_>, DataError> = self
            .variables
            .scan_prefix(prefix)
            .map(|entry| match entry {
                Ok((key, raw_value)) => {
                    //Strips room and username from key, leaving
                    //behind name.
                    let variable_name = std::str::from_utf8(&key[prefix_len..])?;
                    Ok((variable_name.to_owned(), convert_i32(&raw_value)?))
                }
                Err(e) => Err(e.into()),
            })
            .collect();

        //Convert_I32 to hash map. Can we do this in the first mapping
        //step instead? For some reason this is faster.
        variables.map(|entries| entries.into_iter().collect())
    }

    pub async fn get_variable_count(
        &self,
        room_id: &str,
        username: &str,
    ) -> Result<i32, DataError> {
        let key = room_variable_count_key(room_id, username);
        if let Some(raw_value) = self.variables.get(&key)? {
            convert_i32(&raw_value)
        } else {
            Ok(0)
        }
    }

    pub async fn get_user_variable(
        &self,
        room_id: &str,
        username: &str,
        variable_name: &str,
    ) -> Result<i32, DataError> {
        let key = to_key(room_id, username, variable_name);

        if let Some(raw_value) = self.variables.get(&key)? {
            convert_i32(&raw_value)
        } else {
            Err(DataError::KeyDoesNotExist(String::from_utf8(key).unwrap()))
        }
    }

    pub async fn set_user_variable(
        &self,
        room_id: &str,
        username: &str,
        variable_name: &str,
        value: i32,
    ) -> Result<(), DataError> {
        self.variables
            .transaction(|tx| {
                let key = to_key(room_id, username, variable_name);
                let db_value: I32<LittleEndian> = I32::new(value);
                let old_value = tx.insert(key, db_value.as_bytes())?;

                //Only increment variable count on new keys.
                if let None = old_value {
                    match alter_room_variable_count(&tx, room_id, username, 1) {
                        Err(e) => abort(e),
                        _ => Ok(()),
                    }
                } else {
                    Ok(())
                }
            })
            .map_err(|e| e.into())
    }

    pub async fn delete_user_variable(
        &self,
        room_id: &str,
        username: &str,
        variable_name: &str,
    ) -> Result<(), DataError> {
        self.variables
            .transaction(|tx| {
                let key = to_key(room_id, username, variable_name);
                if let Some(_) = tx.remove(key.clone())? {
                    match alter_room_variable_count(&tx, room_id, username, -1) {
                        Err(e) => abort(e),
                        _ => Ok(()),
                    }
                } else {
                    abort(DataError::KeyDoesNotExist(String::from_utf8(key).unwrap()))
                }
            })
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    //Room Variable count tests

    #[tokio::test]
    async fn alter_room_variable_count_test() {
        let db = Database::new(&tempdir().unwrap()).unwrap();

        let alter_count = |amount: i32| {
            db.variables
                .transaction(|tx| {
                    match alter_room_variable_count(&tx, "room", "username", amount) {
                        Err(e) => abort(e),
                        _ => Ok(()),
                    }
                })
                .expect("got transaction failure");
        };

        async fn get_count(db: &Database) -> i32 {
            db.get_variable_count("room", "username")
                .await
                .expect("could not get variable count")
        }

        //addition
        alter_count(5);
        assert_eq!(5, get_count(&db).await);

        //subtraction
        alter_count(-3);
        assert_eq!(2, get_count(&db).await);
    }

    #[tokio::test]
    async fn alter_room_variable_count_cannot_go_below_0_test() {
        let db = Database::new(&tempdir().unwrap()).unwrap();
        db.variables
            .transaction(
                |tx| match alter_room_variable_count(&tx, "room", "username", -1000) {
                    Err(e) => abort(e),
                    _ => Ok(()),
                },
            )
            .expect("got transaction failure");

        let count = db
            .get_variable_count("room", "username")
            .await
            .expect("could not get variable count");

        assert_eq!(0, count);
    }

    #[tokio::test]
    async fn empty_db_reports_0_room_variable_count_test() {
        let db = Database::new(&tempdir().unwrap()).unwrap();

        let count = db
            .get_variable_count("room", "username")
            .await
            .expect("could not get variable count");

        assert_eq!(0, count);
    }

    #[tokio::test]
    async fn set_user_variable_increments_count() {
        let db = Database::new(&tempdir().unwrap()).unwrap();
        db.set_user_variable("room", "username", "myvariable", 5)
            .await
            .expect("could not insert variable");

        let count = db
            .get_variable_count("room", "username")
            .await
            .expect("could not get variable count");

        assert_eq!(1, count);
    }

    #[tokio::test]
    async fn update_user_variable_does_not_increment_count() {
        let db = Database::new(&tempdir().unwrap()).unwrap();
        db.set_user_variable("room", "username", "myvariable", 5)
            .await
            .expect("could not insert variable");

        db.set_user_variable("room", "username", "myvariable", 10)
            .await
            .expect("could not update variable");

        let count = db
            .get_variable_count("room", "username")
            .await
            .expect("could not get variable count");

        assert_eq!(1, count);
    }

    // Set/get/delete variable tests

    #[tokio::test]
    async fn set_and_get_variable_test() {
        let db = Database::new(&tempdir().unwrap()).unwrap();
        db.set_user_variable("room", "username", "myvariable", 5)
            .await
            .expect("could not insert variable");

        let value = db
            .get_user_variable("room", "username", "myvariable")
            .await
            .expect("could not get value");

        assert_eq!(5, value);
    }

    #[tokio::test]
    async fn delete_variable_test() {
        let db = Database::new(&tempdir().unwrap()).unwrap();
        db.set_user_variable("room", "username", "myvariable", 5)
            .await
            .expect("could not insert variable");

        db.delete_user_variable("room", "username", "myvariable")
            .await
            .expect("could not delete value");

        let result = db.get_user_variable("room", "username", "myvariable").await;

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }

    #[tokio::test]
    async fn get_missing_variable_returns_key_does_not_exist() {
        let db = Database::new(&tempdir().unwrap()).unwrap();
        let result = db.get_user_variable("room", "username", "myvariable").await;

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }

    #[tokio::test]
    async fn remove_missing_variable_returns_key_does_not_exist() {
        let db = Database::new(&tempdir().unwrap()).unwrap();
        let result = db
            .delete_user_variable("room", "username", "myvariable")
            .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }
}
