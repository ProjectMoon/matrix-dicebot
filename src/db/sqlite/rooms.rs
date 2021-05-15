use super::errors::DataError;
use super::{Database, Rooms};
use crate::models::RoomInfo;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};

#[async_trait]
impl Rooms for Database {
    async fn should_process(&self, room_id: &str, event_id: &str) -> Result<bool, DataError> {
        Ok(true)
    }

    async fn insert_room_info(&self, info: &RoomInfo) -> Result<(), DataError> {
        Ok(())
    }

    async fn get_room_info(&self, room_id: &str) -> Result<Option<RoomInfo>, DataError> {
        Ok(Some(RoomInfo {
            room_id: "".to_string(),
            room_name: "".to_string(),
        }))
    }

    async fn get_rooms_for_user(&self, user_id: &str) -> Result<HashSet<String>, DataError> {
        Ok(HashSet::new())
    }

    async fn get_users_in_room(&self, room_id: &str) -> Result<HashSet<String>, DataError> {
        Ok(HashSet::new())
    }

    async fn add_user_to_room(&self, username: &str, room_id: &str) -> Result<(), DataError> {
        Ok(())
    }

    async fn remove_user_from_room(&self, username: &str, room_id: &str) -> Result<(), DataError> {
        Ok(())
    }

    async fn clear_info(&self, room_id: &str) -> Result<(), DataError> {
        Ok(())
    }
}
