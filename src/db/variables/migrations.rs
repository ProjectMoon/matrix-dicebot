use super::*;
use crate::db::errors::{DataError, MigrationError};
use crate::db::Database;
use byteorder::LittleEndian;
use memmem::{Searcher, TwoWaySearcher};
use sled::transaction::TransactionError;
use sled::{Batch, IVec};
use std::collections::HashMap;
use zerocopy::byteorder::{I32, U32};
use zerocopy::AsBytes;

pub(in crate::db) mod add_room_user_variable_count {
    use super::*;
    //Not to be confused with the super::RoomAndUser delineator.
    #[derive(PartialEq, Eq, std::hash::Hash)]
    struct RoomAndUser {
        room_id: String,
        username: String,
    }

    /// Create a version 0 user variable key.
    fn v0_variable_key(info: &RoomAndUser, variable_name: &str) -> Vec<u8> {
        let mut key = vec![];
        key.extend_from_slice(info.room_id.as_bytes());
        key.extend_from_slice(info.username.as_bytes());
        key.extend_from_slice(variable_name.as_bytes());
        key
    }

    fn map_value_to_room_and_user(
        entry: sled::Result<(IVec, IVec)>,
    ) -> Result<RoomAndUser, MigrationError> {
        if let Ok((key, _)) = entry {
            let keys: Vec<Result<&str, _>> = key
                .split(|&b| b == 0xff)
                .map(|b| str::from_utf8(b))
                .collect();

            if let &[_, Ok(room_id), Ok(username), Ok(_variable)] = keys.as_slice() {
                Ok(RoomAndUser {
                    room_id: room_id.to_owned(),
                    username: username.to_owned(),
                })
            } else {
                Err(MigrationError::MigrationFailed(
                    "a key violates utf8 schema".to_string(),
                ))
            }
        } else {
            Err(MigrationError::MigrationFailed(
                "encountered unexpected key".to_string(),
            ))
        }
    }

    fn create_key(room_id: &str, username: &str) -> Vec<u8> {
        let mut key = b"variables".to_vec();
        key.push(0xff);
        key.extend_from_slice(room_id.as_bytes());
        key.push(0xff);
        key.extend_from_slice(username.as_bytes());
        key.push(0xff);
        key.extend_from_slice(b"variable_count");
        key
    }

    pub(in crate::db) fn migrate(db: &Database) -> Result<(), DataError> {
        let tree = &db.variables.room_user_variables;
        let prefix = b"variables";

        //Extract a vec of tuples, consisting of room id + username.
        let results: Vec<RoomAndUser> = tree
            .scan_prefix(prefix)
            .map(map_value_to_room_and_user)
            .collect::<Result<Vec<_>, MigrationError>>()?;

        let counts: HashMap<RoomAndUser, u32> =
            results
                .into_iter()
                .fold(HashMap::new(), |mut count_map, room_and_user| {
                    let count = count_map.entry(room_and_user).or_insert(0);
                    *count += 1;
                    count_map
                });

        //Start a transaction on the variables tree.
        let tx_result: Result<_, TransactionError<DataError>> =
            db.variables.room_user_variables.transaction(|tx_vars| {
                let batch = counts.iter().fold(Batch::default(), |mut batch, entry| {
                    let (info, count) = entry;

                    //Add variable count according to new schema.
                    let key = create_key(&info.room_id, &info.username);
                    let db_value: U32<LittleEndian> = U32::new(*count);
                    batch.insert(key, db_value.as_bytes());

                    //Delete the old variable_count variable if exists.
                    let old_key = v0_variable_key(&info, "variable_count");
                    batch.remove(old_key);
                    batch
                });

                tx_vars.apply_batch(&batch)?;
                Ok(())
            });

        tx_result?; //For some reason, it cannot infer the type
        Ok(())
    }
}

pub(in crate::db) fn delete_v0_schema(db: &Database) -> Result<(), DataError> {
    let mut vars = db.variables.room_user_variables.scan_prefix("");
    let mut batch = Batch::default();

    while let Some(Ok((key, _))) = vars.next() {
        let key = key.to_vec();

        if !key.contains(&0xff) {
            batch.remove(key);
        }
    }

    db.variables.room_user_variables.apply_batch(batch)?;
    Ok(())
}

pub(in crate::db) fn delete_variable_count(db: &Database) -> Result<(), DataError> {
    let prefix = b"variables";
    let mut vars = db.variables.room_user_variables.scan_prefix(prefix);
    let mut batch = Batch::default();

    while let Some(Ok((key, _))) = vars.next() {
        let search = TwoWaySearcher::new(b"variable_count");
        let ends_with = {
            match search.search_in(&key) {
                Some(index) => key.len() - index == b"variable_count".len(),
                None => false,
            }
        };

        if ends_with {
            batch.remove(key);
        }
    }

    db.variables.room_user_variables.apply_batch(batch)?;
    Ok(())
}

pub(in crate::db) mod change_delineator_delimiter {
    use super::*;

    /// An entry in the room user variables keyspace.
    struct UserVariableEntry {
        room_id: String,
        username: String,
        variable_name: String,
        value: IVec,
    }

    /// Extract keys and values from the variables keyspace according
    /// to the v1 schema.
    fn extract_v1_entries(
        entry: sled::Result<(IVec, IVec)>,
    ) -> Result<UserVariableEntry, MigrationError> {
        if let Ok((key, value)) = entry {
            let keys: Vec<Result<&str, _>> = key
                .split(|&b| b == 0xff)
                .map(|b| str::from_utf8(b))
                .collect();

            if let &[_, Ok(room_id), Ok(username), Ok(variable)] = keys.as_slice() {
                Ok(UserVariableEntry {
                    room_id: room_id.to_owned(),
                    username: username.to_owned(),
                    variable_name: variable.to_owned(),
                    value: value,
                })
            } else {
                Err(MigrationError::MigrationFailed(
                    "a key violates utf8 schema".to_string(),
                ))
            }
        } else {
            Err(MigrationError::MigrationFailed(
                "encountered unexpected key".to_string(),
            ))
        }
    }

    /// Create an old key, where delineator is separated by 0xff.
    fn create_old_key(prefix: &[u8], insert: &UserVariableEntry) -> Vec<u8> {
        let mut key = vec![];
        key.extend_from_slice(&prefix); //prefix already has 0xff.
        key.extend_from_slice(&insert.room_id.as_bytes());
        key.push(0xff);
        key.extend_from_slice(&insert.username.as_bytes());
        key.push(0xff);
        key.extend_from_slice(&insert.variable_name.as_bytes());
        key
    }

    /// Create an old key, where delineator is separated by 0xfe.
    fn create_new_key(prefix: &[u8], insert: &UserVariableEntry) -> Vec<u8> {
        let mut key = vec![];
        key.extend_from_slice(&prefix); //prefix already has 0xff.
        key.extend_from_slice(&insert.room_id.as_bytes());
        key.push(0xfe);
        key.extend_from_slice(&insert.username.as_bytes());
        key.push(0xff);
        key.extend_from_slice(&insert.variable_name.as_bytes());
        key
    }

    pub fn migrate(db: &Database) -> Result<(), DataError> {
        let tree = &db.variables.room_user_variables;
        let prefix = b"variables";

        let results: Vec<UserVariableEntry> = tree
            .scan_prefix(&prefix)
            .map(extract_v1_entries)
            .collect::<Result<Vec<_>, MigrationError>>()?;

        let mut batch = Batch::default();

        for insert in results {
            let old = create_old_key(prefix, &insert);
            let new = create_new_key(prefix, &insert);

            batch.remove(old);
            batch.insert(new, insert.value);
        }

        tree.apply_batch(batch)?;
        Ok(())
    }
}

/// Move the user variable entries into two tree structures, with yet
/// another key format change. Now there is one tree for variable
/// counts, and one tree for actual user variables. Keys in the user
/// variable tree were changed to be username-first, then room ID.
/// They are still separated by 0xfe, while the variable name is
/// separated by 0xff. Variable count now stores just
/// USERNAME0xfeROOM_ID and a count in its own tree. This enables
/// public use of a strongly typed UserAndRoom struct for getting
/// variables.
pub(in crate::db) mod change_tree_structure {
    use super::*;

    /// An entry in the room user variables keyspace.
    struct UserVariableEntry {
        room_id: String,
        username: String,
        variable_name: String,
        value: IVec,
    }

    /// Extract keys and values from the variables keyspace according
    /// to the v1 schema.
    fn extract_v1_entries(
        entry: sled::Result<(IVec, IVec)>,
    ) -> Result<UserVariableEntry, MigrationError> {
        if let Ok((key, value)) = entry {
            let keys: Vec<Result<&str, _>> = key
                .split(|&b| b == 0xff || b == 0xfe)
                .map(|b| str::from_utf8(b))
                .collect();

            if let &[_, Ok(room_id), Ok(username), Ok(variable)] = keys.as_slice() {
                Ok(UserVariableEntry {
                    room_id: room_id.to_owned(),
                    username: username.to_owned(),
                    variable_name: variable.to_owned(),
                    value: value,
                })
            } else {
                Err(MigrationError::MigrationFailed(
                    "a key violates utf8 schema".to_string(),
                ))
            }
        } else {
            Err(MigrationError::MigrationFailed(
                "encountered unexpected key".to_string(),
            ))
        }
    }

    /// Create an old key, of "variables" 0xff "room id" 0xfe "username" 0xff "variablename".
    fn create_old_key(prefix: &[u8], insert: &UserVariableEntry) -> Vec<u8> {
        let mut key = vec![];
        key.extend_from_slice(&prefix); //prefix already has 0xff.
        key.extend_from_slice(&insert.room_id.as_bytes());
        key.push(0xff);
        key.extend_from_slice(&insert.username.as_bytes());
        key.push(0xff);
        key.extend_from_slice(&insert.variable_name.as_bytes());
        key
    }

    /// Create a new key, of "username" 0xfe "room id" 0xff "variablename".
    fn create_new_key(insert: &UserVariableEntry) -> Vec<u8> {
        let mut key = vec![];
        key.extend_from_slice(&insert.username.as_bytes());
        key.push(0xfe);
        key.extend_from_slice(&insert.room_id.as_bytes());
        key.push(0xff);
        key.extend_from_slice(&insert.variable_name.as_bytes());
        key
    }

    pub fn migrate(db: &Database) -> Result<(), DataError> {
        let variables_tree = &db.variables.room_user_variables;
        let count_tree = &db.variables.room_user_variable_count;
        let prefix = b"variables";

        let results: Vec<UserVariableEntry> = variables_tree
            .scan_prefix(&prefix)
            .map(extract_v1_entries)
            .collect::<Result<Vec<_>, MigrationError>>()?;

        let mut counts: HashMap<(String, String), i32> = HashMap::new();
        let mut batch = Batch::default();

        for insert in results {
            let count = counts
                .entry((insert.username.clone(), insert.room_id.clone()))
                .or_insert(0);
            *count += 1;

            let old = create_old_key(prefix, &insert);
            let new = create_new_key(&insert);

            batch.remove(old);
            batch.insert(new, insert.value);
        }

        let mut count_batch = Batch::default();
        counts.into_iter().for_each(|((username, room_id), count)| {
            let mut key = username.as_bytes().to_vec();
            key.push(0xfe);
            key.extend_from_slice(room_id.as_bytes());

            let db_value: I32<LittleEndian> = I32::new(count);
            count_batch.insert(key, db_value.as_bytes());
        });

        variables_tree.apply_batch(batch)?;
        count_tree.apply_batch(count_batch)?;

        Ok(())
    }
}
