use super::*;
use crate::db::errors::{DataError, MigrationError};
use crate::db::Database;
use byteorder::LittleEndian;
use memmem::{Searcher, TwoWaySearcher};
use sled::transaction::TransactionError;
use sled::{Batch, IVec};
use zerocopy::byteorder::U32;
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

    pub(in crate::db) fn migrate(db: &Database) -> Result<(), DataError> {
        let tree = &db.variables.0;
        let prefix = variables_space_prefix("");

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
            db.variables.0.transaction(|tx_vars| {
                let batch = counts.iter().fold(Batch::default(), |mut batch, entry| {
                    let (info, count) = entry;

                    //Add variable count according to new schema.
                    let delineator = super::RoomAndUser(&info.room_id, &info.username);
                    let key = variables_space_key(delineator, VARIABLE_COUNT_KEY);
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
    let mut vars = db.variables.0.scan_prefix("");
    let mut batch = Batch::default();

    while let Some(Ok((key, _))) = vars.next() {
        let key = key.to_vec();

        if !key.contains(&0xff) {
            batch.remove(key);
        }
    }

    db.variables.0.apply_batch(batch)?;
    Ok(())
}

pub(in crate::db) fn delete_variable_count(db: &Database) -> Result<(), DataError> {
    let prefix = variables_space_prefix("");
    let mut vars = db.variables.0.scan_prefix(prefix);
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

    db.variables.0.apply_batch(batch)?;
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
        let tree = &db.variables.0;
        let prefix = variables_space_prefix("");

        let results: Vec<UserVariableEntry> = tree
            .scan_prefix(&prefix)
            .map(extract_v1_entries)
            .collect::<Result<Vec<_>, MigrationError>>()?;

        let mut batch = Batch::default();

        for insert in results {
            let old = create_old_key(&prefix, &insert);
            let new = create_new_key(&prefix, &insert);

            batch.remove(old);
            batch.insert(new, insert.value);
        }

        tree.apply_batch(batch)?;
        Ok(())
    }
}
