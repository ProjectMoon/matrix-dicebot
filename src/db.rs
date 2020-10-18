use byteorder::LittleEndian;
use sled::{Db, IVec, Tree};
use std::collections::HashMap;
use thiserror::Error;
use zerocopy::byteorder::I32;
use zerocopy::{AsBytes, LayoutVerified};

/// User variables are stored as little-endian 32-bit integers in the
/// database. This type alias makes the database code more pleasant to
/// read.
type LittleEndianI32Layout<'a> = LayoutVerified<&'a [u8], I32<LittleEndian>>;

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
}

fn to_key(room_id: &str, username: &str, variable_name: &str) -> Vec<u8> {
    let mut key = vec![];
    key.extend_from_slice(room_id.as_bytes());
    key.extend_from_slice(username.as_bytes());
    key.extend_from_slice(variable_name.as_bytes());
    key
}

fn to_prefix(room_id: &str, username: &str) -> Vec<u8> {
    let mut prefix = vec![];
    prefix.extend_from_slice(room_id.as_bytes());
    prefix.extend_from_slice(username.as_bytes());
    prefix
}

fn convert(raw_value: &[u8]) -> Result<i32, DataError> {
    let layout = LittleEndianI32Layout::new_unaligned(raw_value.as_ref());

    if let Some(layout) = layout {
        let value: I32<LittleEndian> = *layout;
        Ok(value.get())
    } else {
        Err(DataError::I32SchemaViolation)
    }
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
            .db
            .scan_prefix(prefix)
            .map(|entry| match entry {
                Ok((key, raw_value)) => {
                    //Strips room and username from key, leaving
                    //behind name.
                    let variable_name = std::str::from_utf8(&key[prefix_len..])?;
                    Ok((variable_name.to_owned(), convert(&raw_value)?))
                }
                Err(e) => Err(e.into()),
            })
            .collect();

        //Convert to hash map. Can we do this in the first mapping
        //step instead?
        variables.map(|entries| entries.into_iter().collect())
    }

    pub async fn get_user_variable(
        &self,
        room_id: &str,
        username: &str,
        variable_name: &str,
    ) -> Result<i32, DataError> {
        let key = to_key(room_id, username, variable_name);

        if let Some(raw_value) = self.db.get(&key)? {
            convert(&raw_value)
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
        let key = to_key(room_id, username, variable_name);
        let db_value: I32<LittleEndian> = I32::new(value);
        self.db.insert(&key, IVec::from(db_value.as_bytes()))?;
        Ok(())
    }

    pub async fn delete_user_variable(
        &self,
        room_id: &str,
        username: &str,
        variable_name: &str,
    ) -> Result<(), DataError> {
        let key = to_key(room_id, username, variable_name);
        if let Some(_) = self.db.remove(&key)? {
            Ok(())
        } else {
            Err(DataError::KeyDoesNotExist(String::from_utf8(key).unwrap()))
        }
    }
}
