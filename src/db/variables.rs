use crate::db::errors::DataError;
use crate::db::schema::convert_i32;
use byteorder::LittleEndian;
use sled::transaction::{abort, TransactionalTree};
use sled::Transactional;
use sled::Tree;
use std::collections::HashMap;
use std::convert::From;
use std::str;
use zerocopy::byteorder::I32;
use zerocopy::AsBytes;

pub(super) mod migrations;

#[derive(Clone)]
pub struct Variables {
    //room id + username + variable = i32
    pub(in crate::db) room_user_variables: Tree,

    //room id + username = i32
    pub(in crate::db) room_user_variable_count: Tree,
}

/// Request soemthing by a username and room ID.
pub struct UserAndRoom<'a>(pub &'a str, pub &'a str);

fn to_vec(value: &UserAndRoom<'_>) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.extend_from_slice(value.0.as_bytes());
    bytes.push(0xfe);
    bytes.extend_from_slice(value.1.as_bytes());
    bytes
}

impl From<UserAndRoom<'_>> for Vec<u8> {
    fn from(value: UserAndRoom) -> Vec<u8> {
        to_vec(&value)
    }
}

impl From<&UserAndRoom<'_>> for Vec<u8> {
    fn from(value: &UserAndRoom) -> Vec<u8> {
        to_vec(value)
    }
}

/// Use a transaction to atomically alter the count of variables in
/// the database by the given amount. Count cannot go below 0.
fn alter_room_variable_count(
    room_variable_count: &TransactionalTree,
    user_and_room: &UserAndRoom<'_>,
    amount: i32,
) -> Result<i32, DataError> {
    let key: Vec<u8> = user_and_room.into();

    let mut new_count = match room_variable_count.get(&key)? {
        Some(bytes) => convert_i32(&bytes)? + amount,
        None => amount,
    };

    if new_count < 0 {
        new_count = 0;
    }

    let db_value: I32<LittleEndian> = I32::new(new_count);
    room_variable_count.insert(key, db_value.as_bytes())?;
    Ok(new_count)
}

impl Variables {
    pub(in crate::db) fn new(db: &sled::Db) -> Result<Variables, sled::Error> {
        Ok(Variables {
            room_user_variables: db.open_tree("variables")?,
            room_user_variable_count: db.open_tree("room_user_variable_count")?,
        })
    }

    pub fn get_user_variables(
        &self,
        key: &UserAndRoom<'_>,
    ) -> Result<HashMap<String, i32>, DataError> {
        let mut prefix: Vec<u8> = key.into();
        prefix.push(0xff);
        let prefix_len: usize = prefix.len();

        let variables: Result<Vec<_>, DataError> = self
            .room_user_variables
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

        //Convert I32 to hash map. collect() inferred via return type.
        variables.map(|entries| entries.into_iter().collect())
    }

    pub fn get_variable_count(&self, user_and_room: &UserAndRoom<'_>) -> Result<i32, DataError> {
        let key: Vec<u8> = user_and_room.into();

        match self.room_user_variable_count.get(&key)? {
            Some(raw_value) => convert_i32(&raw_value),
            None => Ok(0),
        }
    }

    pub fn get_user_variable(
        &self,
        user_and_room: &UserAndRoom<'_>,
        variable_name: &str,
    ) -> Result<i32, DataError> {
        let mut key: Vec<u8> = user_and_room.into();
        key.push(0xff);
        key.extend_from_slice(variable_name.as_bytes());

        match self.room_user_variables.get(&key)? {
            Some(raw_value) => convert_i32(&raw_value),
            _ => Err(DataError::KeyDoesNotExist(variable_name.to_owned())),
        }
    }

    pub fn set_user_variable(
        &self,
        user_and_room: &UserAndRoom<'_>,
        variable_name: &str,
        value: i32,
    ) -> Result<(), DataError> {
        if self.get_variable_count(user_and_room)? >= 100 {
            return Err(DataError::TooManyEntries);
        }

        (&self.room_user_variables, &self.room_user_variable_count).transaction(
            |(tx_vars, tx_counts)| {
                let mut key: Vec<u8> = user_and_room.into();
                key.push(0xff);
                key.extend_from_slice(variable_name.as_bytes());

                let db_value: I32<LittleEndian> = I32::new(value);
                let old_value = tx_vars.insert(key, db_value.as_bytes())?;

                //Only increment variable count on new keys.
                if let None = old_value {
                    if let Err(e) = alter_room_variable_count(&tx_counts, &user_and_room, 1) {
                        return abort(e);
                    }
                }

                Ok(())
            },
        )?;

        Ok(())
    }

    pub fn delete_user_variable(
        &self,
        user_and_room: &UserAndRoom<'_>,
        variable_name: &str,
    ) -> Result<(), DataError> {
        (&self.room_user_variables, &self.room_user_variable_count).transaction(
            |(tx_vars, tx_counts)| {
                let mut key: Vec<u8> = user_and_room.into();
                key.push(0xff);
                key.extend_from_slice(variable_name.as_bytes());

                //TODO why does tx.remove require moving the key?
                if let Some(_) = tx_vars.remove(key.clone())? {
                    if let Err(e) = alter_room_variable_count(&tx_counts, user_and_room, -1) {
                        return abort(e);
                    }
                } else {
                    return abort(DataError::KeyDoesNotExist(variable_name.to_owned()));
                }

                Ok(())
            },
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sled::Config;

    fn create_test_instance() -> Variables {
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();
        Variables::new(&db).unwrap()
    }

    //Room Variable count tests

    #[test]
    fn alter_room_variable_count_test() {
        let variables = create_test_instance();
        let key = UserAndRoom("username", "room");

        let alter_count = |amount: i32| {
            variables
                .room_user_variable_count
                .transaction(|tx| match alter_room_variable_count(&tx, &key, amount) {
                    Err(e) => abort(e),
                    _ => Ok(()),
                })
                .expect("got transaction failure");
        };

        let get_count = |variables: &Variables| -> i32 {
            variables
                .get_variable_count(&key)
                .expect("could not get variable count")
        };

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
        let key = UserAndRoom("username", "room");

        variables
            .room_user_variable_count
            .transaction(|tx| match alter_room_variable_count(&tx, &key, -1000) {
                Err(e) => abort(e),
                _ => Ok(()),
            })
            .expect("got transaction failure");

        let count = variables
            .get_variable_count(&key)
            .expect("could not get variable count");

        assert_eq!(0, count);
    }

    #[test]
    fn empty_db_reports_0_room_variable_count_test() {
        let variables = create_test_instance();
        let key = UserAndRoom("username", "room");

        let count = variables
            .get_variable_count(&key)
            .expect("could not get variable count");

        assert_eq!(0, count);
    }

    #[test]
    fn set_user_variable_increments_count() {
        let variables = create_test_instance();
        let key = UserAndRoom("username", "room");

        variables
            .set_user_variable(&key, "myvariable", 5)
            .expect("could not insert variable");

        let count = variables
            .get_variable_count(&key)
            .expect("could not get variable count");

        assert_eq!(1, count);
    }

    #[test]
    fn update_user_variable_does_not_increment_count() {
        let variables = create_test_instance();
        let key = UserAndRoom("username", "room");

        variables
            .set_user_variable(&key, "myvariable", 5)
            .expect("could not insert variable");

        variables
            .set_user_variable(&key, "myvariable", 10)
            .expect("could not update variable");

        let count = variables
            .get_variable_count(&key)
            .expect("could not get variable count");

        assert_eq!(1, count);
    }

    // Set/get/delete variable tests

    #[test]
    fn set_and_get_variable_test() {
        let variables = create_test_instance();
        let key = UserAndRoom("username", "room");

        variables
            .set_user_variable(&key, "myvariable", 5)
            .expect("could not insert variable");

        let value = variables
            .get_user_variable(&key, "myvariable")
            .expect("could not get value");

        assert_eq!(5, value);
    }

    #[test]
    fn cannot_set_more_than_100_variables_per_room() {
        let variables = create_test_instance();
        let key = UserAndRoom("username", "room");

        for c in 0..100 {
            variables
                .set_user_variable(&key, &format!("myvariable{}", c), 5)
                .expect("could not insert variable");
        }

        let result = variables.set_user_variable(&key, "myvariable101", 5);
        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::TooManyEntries)));
    }

    #[test]
    fn delete_variable_test() {
        let variables = create_test_instance();
        let key = UserAndRoom("username", "room");

        variables
            .set_user_variable(&key, "myvariable", 5)
            .expect("could not insert variable");

        variables
            .delete_user_variable(&key, "myvariable")
            .expect("could not delete value");

        let result = variables.get_user_variable(&key, "myvariable");

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }

    #[test]
    fn get_missing_variable_returns_key_does_not_exist() {
        let variables = create_test_instance();
        let key = UserAndRoom("username", "room");
        let result = variables.get_user_variable(&key, "myvariable");

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }

    #[test]
    fn remove_missing_variable_returns_key_does_not_exist() {
        let variables = create_test_instance();
        let key = UserAndRoom("username", "room");
        let result = variables.delete_user_variable(&key, "myvariable");

        assert!(result.is_err());
        assert!(matches!(result, Err(DataError::KeyDoesNotExist(_))));
    }
}
