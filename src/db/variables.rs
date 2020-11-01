use crate::db::errors::DataError;
use crate::db::schema::convert_i32;
use byteorder::LittleEndian;
use sled::transaction::{abort, TransactionalTree};
use sled::Tree;
use std::collections::HashMap;
use std::convert::From;
use std::str;
use zerocopy::byteorder::I32;
use zerocopy::AsBytes;

pub(super) mod migrations;

const METADATA_SPACE: &'static [u8] = b"metadata";
const VARIABLE_SPACE: &'static [u8] = b"variables";

const VARIABLE_COUNT_KEY: &'static str = "variable_count";

#[derive(Clone)]
pub struct Variables(pub(super) Tree);

//TODO at least some of these will probalby move elsewhere.

fn space_prefix<D: Into<Vec<u8>>>(space: &[u8], delineator: D) -> Vec<u8> {
    let mut metadata_prefix = vec![];
    metadata_prefix.extend_from_slice(space);
    metadata_prefix.push(0xff);
    let delineator = delineator.into();

    if delineator.len() > 0 {
        metadata_prefix.extend_from_slice(delineator.as_bytes());
        metadata_prefix.push(0xff);
    }

    metadata_prefix
}

fn metadata_space_prefix<D: Into<Vec<u8>>>(delineator: D) -> Vec<u8> {
    space_prefix(METADATA_SPACE, delineator)
}

fn metadata_space_key<D: Into<Vec<u8>>>(delineator: D, key_name: &str) -> Vec<u8> {
    let mut metadata_key = metadata_space_prefix(delineator);
    metadata_key.extend_from_slice(key_name.as_bytes());
    metadata_key
}

fn variables_space_prefix<D: Into<Vec<u8>>>(delineator: D) -> Vec<u8> {
    space_prefix(VARIABLE_SPACE, delineator)
}

fn variables_space_key<D: Into<Vec<u8>>>(delineator: D, key_name: &str) -> Vec<u8> {
    let mut metadata_key = variables_space_prefix(delineator);
    metadata_key.extend_from_slice(key_name.as_bytes());
    metadata_key
}

/// Delineator for keeping track of a key by room ID and username.
struct RoomAndUser<'a>(&'a str, &'a str);

impl<'a> From<RoomAndUser<'a>> for Vec<u8> {
    fn from(value: RoomAndUser<'a>) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(value.0.as_bytes());
        bytes.push(0xfe);
        bytes.extend_from_slice(value.1.as_bytes());
        bytes
    }
}

/// Use a transaction to atomically alter the count of variables in
/// the database by the given amount. Count cannot go below 0.
fn alter_room_variable_count(
    variables: &TransactionalTree,
    room_id: &str,
    username: &str,
    amount: i32,
) -> Result<i32, DataError> {
    let key = metadata_space_key(RoomAndUser(room_id, username), VARIABLE_COUNT_KEY);

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

impl Variables {
    pub fn get_user_variables(
        &self,
        room_id: &str,
        username: &str,
    ) -> Result<HashMap<String, i32>, DataError> {
        let prefix = variables_space_prefix(RoomAndUser(room_id, username));
        let prefix_len: usize = prefix.len();

        let variables: Result<Vec<_>, DataError> = self
            .0
            .scan_prefix(prefix)
            .map(|entry| match entry {
                Ok((key, raw_value)) => {
                    //Strips room and username from key, leaving behind name.
                    let variable_name = str::from_utf8(&key[prefix_len..])?;
                    Ok((variable_name.to_owned(), convert_i32(&raw_value)?))
                }
                Err(e) => Err(e.into()),
            })
            .collect();

        //Convert I32 to hash map. Can we do this in the first mapping
        //step instead? For some reason this is faster.
        variables.map(|entries| entries.into_iter().collect())
    }

    pub fn get_variable_count(&self, room_id: &str, username: &str) -> Result<i32, DataError> {
        let delineator = RoomAndUser(room_id, username);
        let key = metadata_space_key(delineator, VARIABLE_COUNT_KEY);

        if let Some(raw_value) = self.0.get(&key)? {
            convert_i32(&raw_value)
        } else {
            Ok(0)
        }
    }

    pub fn get_user_variable(
        &self,
        room_id: &str,
        username: &str,
        variable_name: &str,
    ) -> Result<i32, DataError> {
        let key = variables_space_key(RoomAndUser(room_id, username), variable_name);

        if let Some(raw_value) = self.0.get(&key)? {
            convert_i32(&raw_value)
        } else {
            Err(DataError::KeyDoesNotExist(variable_name.to_owned()))
        }
    }

    pub fn set_user_variable(
        &self,
        room_id: &str,
        username: &str,
        variable_name: &str,
        value: i32,
    ) -> Result<(), DataError> {
        self.0
            .transaction(|tx| {
                let key = variables_space_key(RoomAndUser(room_id, username), variable_name);
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

    pub fn delete_user_variable(
        &self,
        room_id: &str,
        username: &str,
        variable_name: &str,
    ) -> Result<(), DataError> {
        self.0
            .transaction(|tx| {
                let key = variables_space_key(RoomAndUser(room_id, username), variable_name);

                //TODO why does tx.remove require moving the key?
                if let Some(_) = tx.remove(key.clone())? {
                    match alter_room_variable_count(&tx, room_id, username, -1) {
                        Err(e) => abort(e),
                        _ => Ok(()),
                    }
                } else {
                    abort(DataError::KeyDoesNotExist(variable_name.to_owned()))
                }
            })
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_instance() -> Variables {
        Variables(
            sled::open(&tempdir().unwrap())
                .unwrap()
                .open_tree("variables")
                .unwrap(),
        )
    }

    //Room Variable count tests

    #[test]
    fn alter_room_variable_count_test() {
        let variables = create_test_instance();

        let alter_count = |amount: i32| {
            variables
                .0
                .transaction(|tx| {
                    match alter_room_variable_count(&tx, "room", "username", amount) {
                        Err(e) => abort(e),
                        _ => Ok(()),
                    }
                })
                .expect("got transaction failure");
        };

        fn get_count(variables: &Variables) -> i32 {
            variables
                .get_variable_count("room", "username")
                .expect("could not get variable count")
        }

        //addition
        alter_count(5);
        assert_eq!(5, get_count(&variables));

        //subtraction
        alter_count(-3);
        assert_eq!(2, get_count(&variables));
    }

    #[test]
    fn alter_room_variable_count_cannot_go_below_0_test() {
        let variables = create_test_instance();

        variables
            .0
            .transaction(
                |tx| match alter_room_variable_count(&tx, "room", "username", -1000) {
                    Err(e) => abort(e),
                    _ => Ok(()),
                },
            )
            .expect("got transaction failure");

        let count = variables
            .get_variable_count("room", "username")
            .expect("could not get variable count");

        assert_eq!(0, count);
    }

    #[test]
    fn empty_db_reports_0_room_variable_count_test() {
        let variables = create_test_instance();

        let count = variables
            .get_variable_count("room", "username")
            .expect("could not get variable count");

        assert_eq!(0, count);
    }

    #[test]
    fn set_user_variable_increments_count() {
        let variables = create_test_instance();

        variables
            .set_user_variable("room", "username", "myvariable", 5)
            .expect("could not insert variable");

        let count = variables
            .get_variable_count("room", "username")
            .expect("could not get variable count");

        assert_eq!(1, count);
    }

    #[test]
    fn update_user_variable_does_not_increment_count() {
        let variables = create_test_instance();

        variables
            .set_user_variable("room", "username", "myvariable", 5)
            .expect("could not insert variable");

        variables
            .set_user_variable("room", "username", "myvariable", 10)
            .expect("could not update variable");

        let count = variables
            .get_variable_count("room", "username")
            .expect("could not get variable count");

        assert_eq!(1, count);
    }

    // Set/get/delete variable tests

    #[test]
    fn set_and_get_variable_test() {
        let variables = create_test_instance();
        variables
            .set_user_variable("room", "username", "myvariable", 5)
            .expect("could not insert variable");

        let value = variables
            .get_user_variable("room", "username", "myvariable")
            .expect("could not get value");

        assert_eq!(5, value);
    }

    #[test]
    fn delete_variable_test() {
        let variables = create_test_instance();

        variables
            .set_user_variable("room", "username", "myvariable", 5)
            .expect("could not insert variable");

        variables
            .delete_user_variable("room", "username", "myvariable")
            .expect("could not delete value");

        let result = variables.get_user_variable("room", "username", "myvariable");

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }

    #[test]
    fn get_missing_variable_returns_key_does_not_exist() {
        let variables = create_test_instance();
        let result = variables.get_user_variable("room", "username", "myvariable");

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }

    #[test]
    fn remove_missing_variable_returns_key_does_not_exist() {
        let variables = create_test_instance();
        let result = variables.delete_user_variable("room", "username", "myvariable");

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }
}
