use super::{Command, Execution};
use crate::context::Context;
use crate::db::errors::DataError;
use crate::logic::record_room_information;
use async_trait::async_trait;
use matrix_sdk::identifiers::UserId;

pub struct ResyncCommand;

type ResyncResult = Result<(), DataError>;

#[async_trait]
impl Command for ResyncCommand {
    fn name(&self) -> &'static str {
        "resync room information"
    }

    async fn execute(&self, ctx: &Context<'_>) -> Execution {
        let our_username: Option<UserId> = ctx.matrix_client.user_id().await;
        let our_username: &str = our_username.as_ref().map_or("", UserId::as_str);

        let result: ResyncResult = record_room_information(
            ctx.matrix_client,
            &ctx.db,
            ctx.room.id,
            &ctx.room.display_name,
            our_username,
        )
        .await;

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
