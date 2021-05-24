use crate::error::BotError;
use crate::models::User;
use async_trait::async_trait;
use errors::DataError;
use std::collections::HashMap;

pub mod errors;
pub mod sqlite;

#[async_trait]
pub(crate) trait DbState {
    async fn get_device_id(&self) -> Result<Option<String>, DataError>;

    async fn set_device_id(&self, device_id: &str) -> Result<(), DataError>;
}

#[async_trait]
pub(crate) trait Users {
    async fn upsert_user(&self, user: &User) -> Result<(), DataError>;

    async fn get_user(&self, username: &str) -> Result<Option<User>, DataError>;

    async fn delete_user(&self, username: &str) -> Result<(), DataError>;

    async fn authenticate_user(
        &self,
        username: &str,
        raw_password: &str,
    ) -> Result<Option<User>, BotError>;
}

#[async_trait]
pub(crate) trait Rooms {
    async fn should_process(&self, room_id: &str, event_id: &str) -> Result<bool, DataError>;
}

// TODO move this up to the top once we delete sled. Traits will be the
// main API, then we can have different impls for different DBs.
#[async_trait]
pub trait Variables {
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
