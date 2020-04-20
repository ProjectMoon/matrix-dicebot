use serde::{self, Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "msgtype")]
#[serde(rename = "m.text")]
pub struct TextMessage {
    body: String,
}

impl TextMessage {
    pub fn body(&self) -> &str {
        &self.body
    }
}

// Need untagged because redactions are blank
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum MessageContent {
    Text(TextMessage),
    Other(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomEvent {
    pub content: MessageContent,
    pub event_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "m.room.message")]
    Room(RoomEvent),

    #[serde(other)]
    Other,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timeline {
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Room {
    pub timeline: Timeline,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rooms {
    pub invite: HashMap<String, serde_json::Value>,
    pub join: HashMap<String, Room>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncCommand {
    pub next_batch: String,
    pub rooms: Rooms,
}
