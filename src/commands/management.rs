use super::{Command, Execution, ExecutionResult};
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

    fn is_secure(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let our_username: Option<UserId> = ctx.matrix_client.user_id().await;
        let our_username: &str = our_username.as_ref().map_or("", UserId::as_str);

        record_room_information(
            ctx.matrix_client,
            &ctx.db,
            ctx.room_id(),
            &ctx.room.display_name,
            our_username,
        )
        .await?;

        let message = "Room information resynced.".to_string();
        Execution::success(message)
    }
}

pub struct RegisterCommand(pub String);

#[async_trait]
impl Command for RegisterCommand {
    fn name(&self) -> &'static str {
        "register user account"
    }

    fn is_secure(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        Execution::success("User account registered".to_string())
    }
}
