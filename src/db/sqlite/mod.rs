use async_trait::async_trait;
use errors::DataError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use sqlx::Connection;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

pub mod errors;
pub mod variables;

// TODO move this up to the top once we delete sled. Traits will be the
// main API, then we can have different impls for different DBs.
#[async_trait]
pub(crate) trait Variables {
    async fn get_user_variables(
        &self,
        user: &str,
        room_id: &str,
    ) -> Result<HashMap<String, i32>, DataError>;

    async fn get_variable_count(&self, user: &str, room_id: &str) -> Result<i32, DataError>;

    async fn get_user_variable(
        &self,
        user: &str,
        room_id: &str,
        variable_name: &str,
    ) -> Result<i32, DataError>;

    async fn set_user_variable(
        &self,
        user: &str,
        room_id: &str,
        variable_name: &str,
        value: i32,
    ) -> Result<(), DataError>;

    async fn delete_user_variable(
        &self,
        user: &str,
        room_id: &str,
        variable_name: &str,
    ) -> Result<(), DataError>;
}

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

        //Return actual conncetion pool.
        let conn = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(path)
            .await?;

        Self::new_db(conn)
    }

    pub async fn new_temp() -> Result<Database, DataError> {
        Self::new("sqlite::memory:").await
    }
}
