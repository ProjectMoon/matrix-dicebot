use super::{Command, Execution};
use crate::context::Context;
use crate::db::errors::DataError;
use crate::matrix;
use async_trait::async_trait;
use matrix_sdk::identifiers::{RoomId, UserId};
use std::convert::TryFrom;

pub struct ResyncCommand;

type ResyncResult = Result<(), DataError>;

#[async_trait]
impl Command for ResyncCommand {
    fn name(&self) -> &'static str {
        "resync room information"
    }

    async fn execute(&self, ctx: &Context<'_>) -> Execution {
        let room_id = RoomId::try_from(ctx.room_id).expect("failed to decode room ID");
        let our_username: Option<UserId> = ctx.matrix_client.user_id().await;
        let our_username: &str = our_username.as_ref().map_or("", UserId::as_str);

        let usernames = matrix::get_users_in_room(&ctx.matrix_client, &room_id).await;

        let result: ResyncResult = usernames
            .into_iter()
            .filter(|username| username != our_username)
            .map(|username| ctx.db.rooms.add_user_to_room(&username, room_id.as_str()))
            .collect(); //Make use of collect impl on Result.

        let (plain, html) = match result {
            Ok(()) => {
                let plain = "Room information resynced".to_string();
                let html = "<p>Room information resynced.</p>".to_string();
                (plain, html)
            }
            Err(e) => {
                let plain = format!("Error: {}", e);
                let html = format!("<p><strong>Error:</strong> {}</p>", e);
                (plain, html)
            }
        };

        Execution { plain, html }
    }
}
