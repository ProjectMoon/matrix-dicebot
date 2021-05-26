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
    pub matrix_client: &'a Client,
    pub room: RoomContext<'a>,
    pub username: &'a str,
    pub message_body: &'a str,
    pub account: Account,
}

impl Context<'_> {
    pub fn room_id(&self) -> &RoomId {
        self.room.id
    }

    pub fn is_secure(&self) -> bool {
        self.room.secure
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
        display_name: String,
        sending_user: &str,
    ) -> Result<RoomContext<'a>, BotError> {
        // TODO is_direct is a hack; should set rooms to Direct
        // Message upon joining, if other contact has requested it.
        // Waiting on SDK support.
        let sending_user = UserId::try_from(sending_user)?;
        let user_in_room = room.get_member(&sending_user).await.ok().is_some();
        let is_direct = room.joined_members().await?.len() == 2;

        Ok(RoomContext {
            id: room.room_id(),
            display_name,
            secure: room.is_encrypted() && is_direct && user_in_room,
        })
    }

    pub async fn new<'a>(
        room: &'a Joined,
        sending_user: &str,
    ) -> Result<RoomContext<'a>, BotError> {
        Self::new_with_name(
            &room,
            room.display_name()
                .await
                .ok()
                .unwrap_or_default()
                .to_string(),
            sending_user,
        )
        .await
    }
}
