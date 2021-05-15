use crate::db::errors::{DataError, MigrationError};
use crate::db::migrations::{get_migration_version, Migrations};
use crate::db::rooms::Rooms;
use crate::db::state::DbState;
use crate::db::variables::Variables;
use log::info;
use sled::{Config, Db};
use std::path::Path;

pub mod data_migrations;
pub mod errors;
pub mod migrations;
pub mod rooms;
pub mod schema;
pub mod sqlite;
pub mod state;
pub mod variables;

#[derive(Clone)]
pub struct Database {
    db: Db,
    pub(crate) variables: Variables,
    pub(crate) migrations: Migrations,
    pub(crate) rooms: Rooms,
    pub(crate) state: DbState,
}

impl Database {
    fn new_db(db: sled::Db) -> Result<Database, DataError> {
        let migrations = db.open_tree("migrations")?;

        let database = Database {
            db: db.clone(),
            variables: Variables::new(&db)?,
            migrations: Migrations(migrations),
            rooms: Rooms::new(&db)?,
            state: DbState::new(&db)?,
        };

        //Start any event handlers.
        database.rooms.start_handler();

        info!("Opened database.");
        Ok(database)
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Database, DataError> {
        let db = sled::open(path)?;
        Self::new_db(db)
    }

    pub fn new_temp() -> Result<Database, DataError> {
        let config = Config::new().temporary(true);
        let db = config.open()?;
        Self::new_db(db)
    }

    pub fn migrate(&self, to_version: u32) -> Result<(), DataError> {
        //get version from db
        let db_version = get_migration_version(&self)?;

        if db_version < to_version {
            info!(
                "Migrating database from version {} to version {}",
                db_version, to_version
            );
            //if db version < to_version, proceed
            //produce range of db_version+1 .. to_version (inclusive)
            let versions_to_run: Vec<u32> = ((db_version + 1)..=to_version).collect();
            let migrations = data_migrations::get_migrations(&versions_to_run)?;

            //execute each closure.
            for (version, migration) in versions_to_run.iter().zip(migrations) {
                let (migration_func, name) = migration;
                //This needs to be transactional on migrations
                //keyspace. abort on migration func error.

                info!("Applying migration {} :: {}", version, name);
                match migration_func(&self) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }?;

                self.migrations.set_migration_version(*version)?;
            }

            info!("Done applying migrations.");
            Ok(())
        } else if db_version > to_version {
            //if db version > to_version, cannot downgrade error
            Err(MigrationError::CannotDowngrade.into())
        } else {
            //if db version == to_version, do nothing
            info!("No database migrations needed.");
            Ok(())
        }
    }
}
