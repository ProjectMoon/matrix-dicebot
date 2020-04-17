use serde::{self, Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "msgtype")]
enum MessageContent {
    #[serde(rename = "m.text")]
    Text { body: String },

    #[serde(other)]
    Other,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "membership")]
enum MemberContent {
    #[serde(rename = "invite")]
    Invite {
        // TODO: maybe leave empty?
        #[serde(default)]
        #[serde(alias = "displayname")]
        display_name: Option<String>,
    },

    #[serde(other)]
    Other,
}

#[derive(Serialize, Deserialize, Debug)]
struct RoomEvent {
    content: MessageContent,
    event_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MemberEvent {
    content: MemberContent,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Event {
    #[serde(rename = "m.room.message")]
    Room(RoomEvent),
    #[serde(rename = "m.room.member")]
    Member(MemberEvent),

    #[serde(other)]
    Other,
}

#[derive(Serialize, Deserialize, Debug)]
struct Timeline {
    events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Room {
    timeline: Timeline,
}

#[derive(Serialize, Deserialize, Debug)]
struct Rooms {
    invite: HashMap<String, serde_json::Value>,
    join: HashMap<String, Room>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SyncCommand {
    next_batch: String,
    rooms: Rooms,
}

async fn sync<S: AsRef<str>>(key: S) -> Result<(), Box<dyn std::error::Error>> {
    let body = reqwest::get(&format!(
        "https://matrix.org/_matrix/client/r0/sync?access_token={}&timeout=3000",
        key.as_ref()
    ))
    .await?
    .text()
    .await?;
    let sync: SyncCommand = serde_json::from_str(&body)?;
    println!("{:#?}", sync);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::args()
        .skip(1)
        .next()
        .expect("Need a key as an argument");
    let mut sigint = signal(SignalKind::interrupt())?;

    loop {
        select! {
            _ = sigint.recv() => {
                break;
            }
            _ = sync(&key) => {
            }
        }
    }

    Ok(())
}
