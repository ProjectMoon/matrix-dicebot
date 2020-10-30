use crate::db::errors::{DataError, MigrationError};
use crate::db::variables::migrations::*;
use crate::db::Database;
use phf::phf_map;

pub(super) type DataMigration = (fn(&Database) -> Result<(), DataError>, &'static str);

static MIGRATIONS: phf::Map<u32, DataMigration> = phf_map! {
    1u32 => (add_room_user_variable_count, "add_room_user_variable_count"),
    2u32 => (delete_v0_schema, "delete_v0_schema"),
    3u32 => (delete_variable_count, "delete_variable_count"),
};

pub fn get_migrations(versions: &[u32]) -> Result<Vec<DataMigration>, MigrationError> {
    let mut migrations: Vec<DataMigration> = vec![];

    for version in versions {
        match MIGRATIONS.get(version) {
            Some(func) => migrations.push(*func),
            None => return Err(MigrationError::MigrationNotFound(*version)),
        }
    }

    Ok(migrations)
}
