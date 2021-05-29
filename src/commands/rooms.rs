use super::{Command, Execution, ExecutionResult};
use crate::context::Context;
use crate::db::Users;
use crate::error::BotError;
use crate::matrix;
use async_trait::async_trait;
use fuse_rust::{Fuse, FuseProperty, Fuseable};
use futures::stream::{self, StreamExt, TryStreamExt};
use matrix_sdk::{identifiers::UserId, Client};
use std::convert::TryFrom;

/// Holds matrix room ID and display name as strings, for use with
/// searching. See search_for_room.
#[derive(Clone, Debug, Eq, PartialEq)]
struct RoomNameAndId {
    id: String,
    name: String,
}

/// Allows searching for a room name and ID struct, instead of just
/// searching room display names directly.
impl Fuseable for RoomNameAndId {
    fn properties(&self) -> Vec<FuseProperty> {
        vec![FuseProperty {
            value: String::from("name"),
            weight: 1.0,
        }]
    }

    fn lookup(&self, key: &str) -> Option<&str> {
        match key {
            "name" => Some(&self.name),
            _ => None,
        }
    }
}

/// Attempt to find a room by either name or Matrix Room ID query
/// string. It prefers the exact room ID first, and then falls back to
/// fuzzy searching based on room display name. The best match is
/// returned, or None if no matches were found.
fn search_for_room<'a>(
    rooms_for_user: &'a [RoomNameAndId],
    search_for: &str,
) -> Option<&'a RoomNameAndId> {
    //Lowest score is the best match.
    let best_fuzzy_match = || -> Option<&RoomNameAndId> {
        Fuse::default()
            .search_text_in_fuse_list(search_for, &rooms_for_user)
            .into_iter()
            .min_by(|r1, r2| r1.score.partial_cmp(&r2.score).unwrap())
            .and_then(|result| rooms_for_user.get(result.index))
    };

    rooms_for_user
        .iter()
        .find(|room| room.id == search_for)
        .or_else(best_fuzzy_match)
}

async fn get_rooms_for_user(
    client: &Client,
    user_id: &str,
) -> Result<Vec<RoomNameAndId>, BotError> {
    let user_id = UserId::try_from(user_id)?;
    let rooms_for_user = matrix::get_rooms_for_user(client, &user_id).await?;
    let mut rooms_for_user: Vec<RoomNameAndId> = stream::iter(rooms_for_user)
        .filter_map(|room| async move {
            Some(room.display_name().await.map(|room_name| RoomNameAndId {
                id: room.room_id().to_string(),
                name: room_name,
            }))
        })
        .try_collect()
        .await?;

    //Alphabetically descending, symbols first, ignore case.
    let sort = |r1: &RoomNameAndId, r2: &RoomNameAndId| {
        r1.name.to_lowercase().cmp(&r2.name.to_lowercase())
    };

    rooms_for_user.sort_by(sort);
    Ok(rooms_for_user)
}

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
        "list rooms"
    }

    fn is_secure(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let rooms_for_user: Vec<String> = get_rooms_for_user(ctx.matrix_client, ctx.username)
            .await
            .map(|rooms| {
                rooms
                    .into_iter()
                    .map(|room| format!("  {}  |  {}", room.id, room.name))
                    .collect()
            })?;

        let html = format!("<pre>{}</pre>", rooms_for_user.join("\n"));
        Execution::success(html)
    }
}

pub struct SetRoomCommand(String);

impl From<SetRoomCommand> for Box<dyn Command> {
    fn from(cmd: SetRoomCommand) -> Self {
        Box::new(cmd)
    }
}

impl TryFrom<String> for SetRoomCommand {
    type Error = BotError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        Ok(SetRoomCommand(input))
    }
}

#[async_trait]
impl Command for SetRoomCommand {
    fn name(&self) -> &'static str {
        "set active room"
    }

    fn is_secure(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        if !ctx.account.is_registered() {
            return Err(BotError::AccountDoesNotExist);
        }

        let rooms_for_user = get_rooms_for_user(ctx.matrix_client, ctx.username).await?;
        let room = search_for_room(&rooms_for_user, &self.0);

        if let Some(room) = room {
            let mut new_user = ctx
                .account
                .registered_user()
                .cloned()
                .ok_or(BotError::AccountDoesNotExist)?;

            new_user.active_room = Some(room.id.clone());
            ctx.db.upsert_user(&new_user).await?;
            Execution::success(format!(r#"Active room set to "{}""#, room.name))
        } else {
            Err(BotError::RoomDoesNotExist)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn set_room_prefers_room_id_over_name() {
        let rooms = vec![
            RoomNameAndId {
                id: "roomid".to_string(),
                name: "room_name".to_string(),
            },
            RoomNameAndId {
                id: "anotherone".to_string(),
                name: "roomid".to_string(),
            },
        ];

        let found_room = search_for_room(&rooms, "roomid");

        assert!(found_room.is_some());
        assert_eq!(found_room.unwrap(), &rooms[0]);
    }
}
