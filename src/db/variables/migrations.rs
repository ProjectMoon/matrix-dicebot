use super::*;
use crate::db::errors::{DataError, MigrationError};
use crate::db::Database;
use byteorder::LittleEndian;
use sled::transaction::TransactionError;
use sled::Batch;
use zerocopy::byteorder::U32;
use zerocopy::AsBytes;

//TODO we will make this set variable count properly.
pub(in crate::db) fn migration1(db: &Database) -> Result<(), DataError> {
    let tree = &db.variables.0;
    let prefix = variables_space_prefix("");

    //Extract a vec of tuples, consisting of room id + username.
    let results: Vec<(String, String)> = tree
        .scan_prefix(prefix)
        .map(|entry| {
            if let Ok((key, _)) = entry {
                let keys: Vec<Result<&str, _>> = key
                    .split(|&b| b == 0xff)
                    .map(|b| str::from_utf8(b))
                    .collect();

                if let &[_, Ok(room_id), Ok(username), Ok(_variable)] = keys.as_slice() {
                    Ok((room_id.to_owned(), username.to_owned()))
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
        })
        .collect::<Result<Vec<_>, MigrationError>>()?;

    let counts: HashMap<(String, String), u32> =
        results
            .into_iter()
            .fold(HashMap::new(), |mut count_map, room_and_user| {
                let count = count_map.entry(room_and_user).or_insert(0);
                *count += 1;
                count_map
            });

    //Start a transaction on the variables tree.
    //Delete the old variable_count variable if exists.
    //Add variable count according to new schema.
    let tx_result: Result<_, TransactionError<DataError>> = db.variables.0.transaction(|tx_vars| {
        let batch = counts.iter().fold(Batch::default(), |mut batch, entry| {
            let key =
                variables_space_key(RoomAndUser(&(entry.0).0, &(entry.0).1), VARIABLE_COUNT_KEY);

            let db_value: U32<LittleEndian> = U32::new(*entry.1);
            batch.insert(key, db_value.as_bytes());
            batch
        });

        tx_vars.apply_batch(&batch)?;
        Ok(())
    });

    tx_result?; //For some reason, it cannot infer the type
    Ok(())
}
