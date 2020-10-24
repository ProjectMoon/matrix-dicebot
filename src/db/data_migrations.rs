use crate::db::errors::{DataError, MigrationError};
use crate::db::Database;
use phf::phf_map;

pub(super) type DataMigration = fn(&Database) -> Result<(), DataError>;

static MIGRATIONS: phf::Map<u32, DataMigration> = phf_map! {
    1u32 => super::variables::migrations::migration1,
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
