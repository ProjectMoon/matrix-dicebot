use serde::{Deserialize, Serialize};

/// RoomInfo has basic metadata about a room: its name, ID, etc.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RoomInfo {
    pub room_id: String,
    pub room_name: String,
}

#[derive(Eq, PartialEq, Debug)]
pub struct User {
    pub username: String,
    pub password: String,
}

impl User {
    pub fn verify_password(&self, raw_password: &str) -> bool {
        argon2::verify_encoded(&self.password, raw_password.as_bytes()).unwrap_or(false)
    }
}
