/**
 * In addition to the terms of the AGPL, portions of this file
 * are governed by the terms of the MIT license, from the Rust Matrix
 * SDK example code.
 */
use super::DiceBot;
use crate::db::sqlite::Database;
use crate::db::Rooms;
use crate::error::BotError;
use async_trait::async_trait;
use log::{debug, error, info, warn};
use matrix_sdk::{
    self,
    events::{
        room::member::MemberEventContent,
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        StrippedStateEvent, SyncMessageEvent,
    },
    room::Room,
    EventHandler,
};
use std::ops::Sub;
use std::time::{Duration, SystemTime};
use std::{clone::Clone, time::UNIX_EPOCH};

/// Check if a message is recent enough to actually process. If the
/// message is within "oldest_message_age" seconds, this function
/// returns true. If it's older than that, it returns false and logs a
/// debug message.
fn check_message_age(
    event: &SyncMessageEvent<MessageEventContent>,
    oldest_message_age: u64,
) -> bool {
    let sending_time = event
        .origin_server_ts
        .to_system_time()
        .unwrap_or(UNIX_EPOCH);

    let oldest_timestamp = SystemTime::now().sub(Duration::from_secs(oldest_message_age));

    if sending_time > oldest_timestamp {
        true
    } else {
        let age = match oldest_timestamp.duration_since(sending_time) {
            Ok(n) => format!("{} seconds too old", n.as_secs()),
            Err(_) => "before the UNIX epoch".to_owned(),
        };

        debug!("Ignoring message because it is {}: {:?}", age, event);
        false
    }
}

/// Determine whether or not to process a received message. This check
/// is necessary in addition to the event processing check because we
/// may receive message events when entering a room for the first
/// time, and we don't want to respond to things before the bot was in
/// the channel, but we do want to respond to things that were sent if
/// the bot left and rejoined quickly.
async fn should_process_message<'a>(
    bot: &DiceBot,
    event: &SyncMessageEvent<MessageEventContent>,
) -> Result<(String, String), BotError> {
    //Ignore messages that are older than configured duration.
    if !check_message_age(event, bot.config.oldest_message_age()) {
        let state_check = bot.state.read().unwrap();
        if !((*state_check).logged_skipped_old_messages()) {
            drop(state_check);
            let mut state = bot.state.write().unwrap();
            (*state).skipped_old_messages();
        }

        return Err(BotError::ShouldNotProcessError);
    }

    let (msg_body, sender_username) = if let SyncMessageEvent {
        content:
            MessageEventContent {
                msgtype: MessageType::Text(TextMessageEventContent { body, .. }),
                ..
            },
        sender,
        ..
    } = event
    {
        (
            body.clone(),
            format!("@{}:{}", sender.localpart(), sender.server_name()),
        )
    } else {
        (String::new(), String::new())
    };

    Ok((msg_body, sender_username))
}

async fn should_process_event(db: &Database, room_id: &str, event_id: &str) -> bool {
    db.should_process(room_id, event_id)
        .await
        .unwrap_or_else(|e| {
            error!(
                "Database error when checking if we should process an event: {}",
                e.to_string()
            );
            false
        })
}

/// This event emitter listens for messages with dice rolling commands.
/// Originally adapted from the matrix-rust-sdk examples.
#[async_trait]
impl EventHandler for DiceBot {
    async fn on_stripped_state_member(
        &self,
        room: Room,
        event: &StrippedStateEvent<MemberEventContent>,
        _: Option<MemberEventContent>,
    ) {
        let room = match room {
            Room::Invited(invited_room) => invited_room,
            _ => return,
        };

        if room.own_user_id().as_str() != event.state_key {
            return;
        }

        info!(
            "Autojoining room {}",
            room.display_name().await.ok().unwrap_or_default()
        );

        if let Err(e) = self.client.join_room_by_id(&room.room_id()).await {
            warn!("Could not join room: {}", e.to_string())
        }
    }

    async fn on_room_message(&self, room: Room, event: &SyncMessageEvent<MessageEventContent>) {
        let room = match room {
            Room::Joined(joined_room) => joined_room,
            _ => return,
        };

        let room_id = room.room_id().as_str();
        if !should_process_event(&self.db, room_id, event.event_id.as_str()).await {
            return;
        }

        let (msg_body, sender_username) =
            if let Ok((msg_body, sender_username)) = should_process_message(self, &event).await {
                (msg_body, sender_username)
            } else {
                return;
            };

        let results = self
            .execute_commands(&room, &sender_username, &msg_body)
            .await;

        self.handle_results(&room, &sender_username, event.event_id.clone(), results)
            .await;
    }
}
