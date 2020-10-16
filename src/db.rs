use byteorder::LittleEndian;
use sled::{Db, IVec};
use thiserror::Error;
use zerocopy::byteorder::I32;
use zerocopy::{AsBytes, LayoutVerified};

#[derive(Clone)]
pub struct Database {
    db: Db,
}

#[derive(Error, Debug)]
pub enum DataError {
    #[error("value does not exist for key: {0}")]
    KeyDoesNotExist(String),

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

impl Database {
    pub fn new(db: &Db) -> Database {
        Database { db: db.clone() }
    }

    pub fn get_user_variable(
        &self,
        room_id: &str,
        username: &str,
        variable_name: &str,
    ) -> Result<i32, DataError> {
        let key = to_key(room_id, username, variable_name);

        if let Some(raw_value) = self.db.get(&key)? {
            let layout: LayoutVerified<&[u8], I32<LittleEndian>> =
                LayoutVerified::new_unaligned(&*raw_value).expect("bytes do not fit schema");

            let value: I32<LittleEndian> = *layout;
            Ok(value.get())
        } else {
            Err(DataError::KeyDoesNotExist(String::from_utf8(key).unwrap()))
        }
    }

    pub fn set_user_variable(
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

    pub fn delete_user_variable(
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
