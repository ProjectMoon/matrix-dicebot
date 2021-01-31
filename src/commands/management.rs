use super::{Command, CommandResult, Execution};
use crate::context::Context;
use crate::logic::record_room_information;
use async_trait::async_trait;
use matrix_sdk::identifiers::UserId;

pub struct ResyncCommand;

#[async_trait]
impl Command for ResyncCommand {
    fn name(&self) -> &'static str {
        "resync room information"
    }

    async fn execute(&self, ctx: &Context<'_>) -> CommandResult {
        let our_username: Option<UserId> = ctx.matrix_client.user_id().await;
        let our_username: &str = our_username.as_ref().map_or("", UserId::as_str);

        record_room_information(
            ctx.matrix_client,
            &ctx.db,
            ctx.room.id,
            &ctx.room.display_name,
            our_username,
        )
        .await?;

        let message = "Room information resynced.".to_string();
        Execution::new(message)
    }
}
