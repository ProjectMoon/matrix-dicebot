use async_trait::async_trait;
use errors::DataError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use std::clone::Clone;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use crate::models::RoomInfo;

pub mod errors;
pub mod rooms;
pub mod state;
pub mod variables;

#[async_trait]
pub(crate) trait DbState {
    async fn get_device_id(&self) -> Result<Option<String>, DataError>;

    async fn set_device_id(&self, device_id: &str) -> Result<(), DataError>;
}

#[async_trait]
pub(crate) trait Rooms {
    async fn should_process(&self, room_id: &str, event_id: &str) -> Result<bool, DataError>;

    async fn insert_room_info(&self, info: &RoomInfo) -> Result<(), DataError>;

    async fn get_room_info(&self, room_id: &str) -> Result<Option<RoomInfo>, DataError>;

    async fn get_rooms_for_user(&self, user_id: &str) -> Result<HashSet<String>, DataError>;

    async fn get_users_in_room(&self, room_id: &str) -> Result<HashSet<String>, DataError>;

    async fn add_user_to_room(&self, username: &str, room_id: &str) -> Result<(), DataError>;

    async fn remove_user_from_room(&self, username: &str, room_id: &str) -> Result<(), DataError>;

    async fn clear_info(&self, room_id: &str) -> Result<(), DataError>;
}

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

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            conn: self.conn.clone(),
        }
    }
}
