use super::{Command, Execution, ExecutionResult};
use crate::context::Context;
use crate::error::BotError;
use async_trait::async_trait;
use futures::stream::{self, StreamExt, TryStreamExt};
use matrix_sdk::identifiers::UserId;
use std::convert::TryFrom;

pub struct ListRoomsCommand;

impl From<ListRoomsCommand> for Box<dyn Command> {
    fn from(cmd: ListRoomsCommand) -> Self {
        Box::new(cmd)
    }
}

impl TryFrom<String> for ListRoomsCommand {
    type Error = BotError;

    fn try_from(_: String) -> Result<Self, Self::Error> {
        Ok(ListRoomsCommand)
    }
}

#[async_trait]
impl Command for ListRoomsCommand {
    fn name(&self) -> &'static str {
        "list rooms command"
    }

    fn is_secure(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let user_id = UserId::try_from(ctx.username)?;
        let rooms_for_user = crate::matrix::get_rooms_for_user(ctx.matrix_client, &user_id).await?;

        let rooms_for_user: Vec<String> = stream::iter(rooms_for_user)
            .filter_map(|room| async move {
                Some(
                    room.display_name()
                        .await
                        .map(|room_name| (room.room_id().to_string(), room_name)),
                )
            })
            .map_ok(|(room_id, room_name)| format!("[{}] {}", room_id, room_name))
            .try_collect()
            .await?;

        let html = format!("{}", rooms_for_user.join("\n"));
        Execution::success(html)
    }
}
