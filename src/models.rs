use serde::{Deserialize, Serialize};

/// RoomInfo has basic metadata about a room: its name, ID, etc.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RoomInfo {
    pub room_id: String,
    pub room_name: String,
}
