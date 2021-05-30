use crate::db::sqlite::Database;
use crate::error::BotError;
use crate::models::Account;
use matrix_sdk::identifiers::{RoomId, UserId};
use matrix_sdk::room::Joined;
use matrix_sdk::Client;
use std::convert::TryFrom;

/// A context carried through the system providing access to things
/// like the database.
#[derive(Clone)]
pub struct Context<'a> {
    pub db: Database,
    pub matrix_client: Client,
    pub origin_room: RoomContext<'a>,
    pub active_room: RoomContext<'a>,
    pub username: &'a str,
    pub message_body: &'a str,
    pub account: Account,
}

impl Context<'_> {
    pub fn active_room_id(&self) -> &RoomId {
        self.active_room.id
    }

    pub fn room_id(&self) -> &RoomId {
        self.origin_room.id
    }

    pub fn is_secure(&self) -> bool {
        self.origin_room.secure
    }
}

#[derive(Clone)]
pub struct RoomContext<'a> {
    pub id: &'a RoomId,
    pub display_name: String,
    pub secure: bool,
}

impl RoomContext<'_> {
    pub async fn new_with_name<'a>(
        room: &'a Joined,
        sending_user: &str,
    ) -> Result<RoomContext<'a>, BotError> {
        // TODO is_direct is a hack; the bot should set eligible rooms
        // to Direct Message upon joining, if other contact has
        // requested it. Waiting on SDK support.
        let display_name = room
            .display_name()
            .await
            .ok()
            .unwrap_or_default()
            .to_string();

        let sending_user = UserId::try_from(sending_user)?;
        let user_in_room = room.get_member(&sending_user).await.ok().is_some();
        let is_direct = room.active_members().await?.len() == 2;

        Ok(RoomContext {
            id: room.room_id(),
            display_name,
            secure: room.is_encrypted() && is_direct && user_in_room,
        })
    }

    pub async fn new<'a>(
        room: &'a Joined,
        sending_user: &'a str,
    ) -> Result<RoomContext<'a>, BotError> {
        Self::new_with_name(room, sending_user).await
    }
}
