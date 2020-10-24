use crate::db::errors::DataError;
use crate::db::schema::convert_i32;
use byteorder::LittleEndian;
use sled::transaction::abort;
use sled::transaction::TransactionalTree;
use sled::Tree;
use std::collections::HashMap;
use std::str;
use zerocopy::byteorder::I32;
use zerocopy::AsBytes;

const METADATA_KEY: &'static str = "metadata";
const VARIABLE_COUNT_KEY: &'static str = "variable_count";

#[derive(Clone)]
pub struct Variables(pub(crate) Tree);

fn to_key(room_id: &str, username: &str, variable_name: &str) -> Vec<u8> {
    let mut key = vec![];
    key.extend_from_slice(room_id.as_bytes());
    key.extend_from_slice(username.as_bytes());
    key.extend_from_slice(variable_name.as_bytes());
    key
}

fn metadata_key(room_id: &str, username: &str, metadata_key: &str) -> Vec<u8> {
    let mut key = vec![];
    key.extend_from_slice(room_id.as_bytes());
    key.extend_from_slice(METADATA_KEY.as_bytes());
    key.extend_from_slice(username.as_bytes());
    key.extend_from_slice(metadata_key.as_bytes());
    key
}

fn room_variable_count_key(room_id: &str, username: &str) -> Vec<u8> {
    metadata_key(room_id, username, VARIABLE_COUNT_KEY)
}

fn to_prefix(room_id: &str, username: &str) -> Vec<u8> {
    let mut prefix = vec![];
    prefix.extend_from_slice(room_id.as_bytes());
    prefix.extend_from_slice(username.as_bytes());
    prefix
}

/// Use a transaction to atomically alter the count of variables in
/// the database by the given amount. Count cannot go below 0.
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

impl Variables {
    pub async fn get_user_variables(
        &self,
        room_id: &str,
        username: &str,
    ) -> Result<HashMap<String, i32>, DataError> {
        let prefix = to_prefix(&room_id, &username);
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

    pub async fn get_variable_count(
        &self,
        room_id: &str,
        username: &str,
    ) -> Result<i32, DataError> {
        let key = room_variable_count_key(room_id, username);
        if let Some(raw_value) = self.0.get(&key)? {
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

        if let Some(raw_value) = self.0.get(&key)? {
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
        self.0
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
        self.0
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

    fn create_test_instance() -> Variables {
        Variables(
            sled::open(&tempdir().unwrap())
                .unwrap()
                .open_tree("variables")
                .unwrap(),
        )
    }

    //Room Variable count tests

    #[tokio::test]
    async fn alter_room_variable_count_test() {
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

        async fn get_count(variables: &Variables) -> i32 {
            variables
                .get_variable_count("room", "username")
                .await
                .expect("could not get variable count")
        }

        //addition
        alter_count(5);
        assert_eq!(5, get_count(&variables).await);

        //subtraction
        alter_count(-3);
        assert_eq!(2, get_count(&variables).await);
    }

    #[tokio::test]
    async fn alter_room_variable_count_cannot_go_below_0_test() {
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
            .await
            .expect("could not get variable count");

        assert_eq!(0, count);
    }

    #[tokio::test]
    async fn empty_db_reports_0_room_variable_count_test() {
        let variables = create_test_instance();

        let count = variables
            .get_variable_count("room", "username")
            .await
            .expect("could not get variable count");

        assert_eq!(0, count);
    }

    #[tokio::test]
    async fn set_user_variable_increments_count() {
        let variables = create_test_instance();

        variables
            .set_user_variable("room", "username", "myvariable", 5)
            .await
            .expect("could not insert variable");

        let count = variables
            .get_variable_count("room", "username")
            .await
            .expect("could not get variable count");

        assert_eq!(1, count);
    }

    #[tokio::test]
    async fn update_user_variable_does_not_increment_count() {
        let variables = create_test_instance();

        variables
            .set_user_variable("room", "username", "myvariable", 5)
            .await
            .expect("could not insert variable");

        variables
            .set_user_variable("room", "username", "myvariable", 10)
            .await
            .expect("could not update variable");

        let count = variables
            .get_variable_count("room", "username")
            .await
            .expect("could not get variable count");

        assert_eq!(1, count);
    }

    // Set/get/delete variable tests

    #[tokio::test]
    async fn set_and_get_variable_test() {
        let variables = create_test_instance();
        variables
            .set_user_variable("room", "username", "myvariable", 5)
            .await
            .expect("could not insert variable");

        let value = variables
            .get_user_variable("room", "username", "myvariable")
            .await
            .expect("could not get value");

        assert_eq!(5, value);
    }

    #[tokio::test]
    async fn delete_variable_test() {
        let variables = create_test_instance();

        variables
            .set_user_variable("room", "username", "myvariable", 5)
            .await
            .expect("could not insert variable");

        variables
            .delete_user_variable("room", "username", "myvariable")
            .await
            .expect("could not delete value");

        let result = variables
            .get_user_variable("room", "username", "myvariable")
            .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }

    #[tokio::test]
    async fn get_missing_variable_returns_key_does_not_exist() {
        let variables = create_test_instance();
        let result = variables
            .get_user_variable("room", "username", "myvariable")
            .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }

    #[tokio::test]
    async fn remove_missing_variable_returns_key_does_not_exist() {
        let variables = create_test_instance();
        let result = variables
            .delete_user_variable("room", "username", "myvariable")
            .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }
}
