use crate::db::errors::DataError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use std::clone::Clone;
use std::str::FromStr;

pub mod migrator;
pub mod rooms;
pub mod state;
pub mod variables;

pub struct Database {
    conn: SqlitePool,
}

impl Database {
    fn new_db(conn: SqlitePool) -> Result<Database, DataError> {
        let database = Database { conn: conn.clone() };
        Ok(database)
    }

    pub async fn new(path: &str) -> Result<Database, DataError> {
        //Create database if missing.
        let conn = SqliteConnectOptions::from_str(path)?
            .create_if_missing(true)
            .connect()
            .await?;

        drop(conn);

        //Migrate database.
        migrator::migrate(&path).await?;

        //Return actual conncetion pool.
        let conn = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(path)
            .await?;

        Self::new_db(conn)
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            conn: self.conn.clone(),
        }
    }
}
