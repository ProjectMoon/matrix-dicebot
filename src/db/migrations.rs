use crate::db::errors::DataError;
use crate::db::schema::convert_u32;
use crate::db::Database;
use byteorder::LittleEndian;
use sled::Tree;
use zerocopy::byteorder::U32;
use zerocopy::AsBytes;

//This file is for controlling the migration info stored in the
//database, not actually running migrations.

#[derive(Clone)]
pub struct Migrations(pub(super) Tree);

const COLON: &'static [u8] = b":";
const METADATA_SPACE: &'static str = "metadata";
const MIGRATION_KEY: &'static str = "migration_version";

fn to_key(keyspace: &str, key_name: &str) -> Vec<u8> {
    let mut key = vec![];
    key.extend_from_slice(keyspace.as_bytes());
    key.extend_from_slice(COLON);
    key.extend_from_slice(key_name.as_bytes());
    key
}

fn metadata_key(key_name: &str) -> Vec<u8> {
    to_key(METADATA_SPACE, key_name)
}

impl Migrations {
    pub(super) fn set_migration_version(&self, version: u32) -> Result<(), DataError> {
        //Rust cannot type infer this transaction
        let result: Result<_, sled::transaction::TransactionError<DataError>> =
            self.0.transaction(|tx| {
                let key = metadata_key(MIGRATION_KEY);
                let db_value: U32<LittleEndian> = U32::new(version);
                tx.insert(key, db_value.as_bytes())?;
                Ok(())
            });

        result?;

        Ok(())
    }
}

pub(super) fn get_migration_version(db: &Database) -> Result<u32, DataError> {
    let key = metadata_key(MIGRATION_KEY);
    match db.migrations.0.get(key)? {
        Some(bytes) => convert_u32(&bytes),
        None => Ok(0),
    }
}
