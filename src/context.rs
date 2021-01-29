use crate::db::Database;
use matrix_sdk::identifiers::RoomId;
use matrix_sdk::Client;
use matrix_sdk::JoinedRoom;

/// A context carried through the system providing access to things
/// like the database.
#[derive(Clone)]
pub struct Context<'a> {
    pub db: Database,
    pub matrix_client: &'a Client,
    pub room: RoomContext<'a>,
    pub username: &'a str,
    pub message_body: &'a str,
}

#[derive(Clone)]
pub struct RoomContext<'a> {
    pub id: &'a RoomId,
    pub display_name: &'a str,
}

impl RoomContext<'_> {
    pub fn new_with_name<'a>(room: &'a JoinedRoom, display_name: &'a str) -> RoomContext<'a> {
        RoomContext {
            id: room.room_id(),
            display_name,
        }
    }
}
