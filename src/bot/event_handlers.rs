use super::DiceBot;
use crate::db::Database;
use crate::error::BotError;
use crate::matrix;
use async_trait::async_trait;
use log::{debug, error, info, warn};
use matrix_sdk::{
    self,
    events::{
        room::member::{MemberEventContent, MembershipChange},
        room::message::{MessageEventContent, TextMessageEventContent},
        StrippedStateEvent, SyncMessageEvent, SyncStateEvent,
    },
    EventEmitter, SyncRoom,
};
use std::clone::Clone;
use std::ops::Sub;
use std::time::{Duration, SystemTime};

/// Check if a message is recent enough to actually process. If the
/// message is within "oldest_message_age" seconds, this function
/// returns true. If it's older than that, it returns false and logs a
/// debug message.
fn check_message_age(
    event: &SyncMessageEvent<MessageEventContent>,
    oldest_message_age: u64,
) -> bool {
    let sending_time = event.origin_server_ts;
    let oldest_timestamp = SystemTime::now().sub(Duration::new(oldest_message_age, 0));

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
        content: MessageEventContent::Text(TextMessageEventContent { body, .. }),
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

fn should_process_event(db: &Database, room_id: &str, event_id: &str) -> bool {
    db.rooms
        .should_process(room_id, event_id)
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
impl EventEmitter for DiceBot {
    async fn on_room_member(&self, room: SyncRoom, event: &SyncStateEvent<MemberEventContent>) {
        if let SyncRoom::Joined(room) | SyncRoom::Left(room) = room {
            //Clone to avoid holding lock.
            let room = room.read().await.clone();
            let (room_id, username) = (room.room_id.as_str(), &event.state_key);

            if !should_process_event(&self.db, room_id, event.event_id.as_str()) {
                return;
            }

            let event_affects_us = if let Some(our_user_id) = self.client.user_id().await {
                event.state_key == our_user_id
            } else {
                false
            };

            use MembershipChange::*;
            let adding_user = match event.membership_change() {
                Joined => true,
                Banned | Left | Kicked | KickedAndBanned => false,
                _ => return,
            };

            let result = if event_affects_us && !adding_user {
                info!("Clearing all information for room ID {}", room_id);
                self.db.rooms.clear_info(room_id)
            } else if event_affects_us && adding_user {
                info!("Joined room {}; recording user information", room_id);
                let usernames = matrix::get_users_in_room(&self.client, &room.room_id).await;
                usernames
                    .into_iter()
                    .filter(|username| username != &event.state_key)
                    .map(|username| self.db.rooms.add_user_to_room(&username, room_id))
                    .collect() //Make use of collect impl on Result.
            } else if !event_affects_us && adding_user {
                info!("Adding user {} to room ID {}", username, room_id);
                self.db.rooms.add_user_to_room(username, room_id)
            } else if !event_affects_us && !adding_user {
                info!("Removing user {} from room ID {}", username, room_id);
                self.db.rooms.remove_user_from_room(username, room_id)
            } else {
                debug!("Ignoring a room member event: {:#?}", event);
                Ok(())
            };

            if let Err(e) = result {
                error!("Could not update room information: {}", e.to_string());
            } else {
                debug!("Successfully processed room member update.");
            }
        }
    }

    async fn on_stripped_state_member(
        &self,
        room: SyncRoom,
        event: &StrippedStateEvent<MemberEventContent>,
        _: Option<MemberEventContent>,
    ) {
        if let SyncRoom::Invited(room) = room {
            if let Some(user_id) = self.client.user_id().await {
                if event.state_key != user_id {
                    return;
                }
            }

            //Clone to avoid holding lock.
            let room = room.read().await.clone();
            info!("Autojoining room {}", room.display_name());

            if let Err(e) = self.client.join_room_by_id(&room.room_id).await {
                warn!("Could not join room: {}", e.to_string())
            }
        }
    }

    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        if let SyncRoom::Joined(room) = room {
            //Clone to avoid holding lock.
            let room = room.read().await.clone();
            let room_id = room.room_id.as_str();
            if !should_process_event(&self.db, room_id, event.event_id.as_str()) {
                return;
            }

            let (msg_body, sender_username) = if let Ok((msg_body, sender_username)) =
                should_process_message(self, &event).await
            {
                (msg_body, sender_username)
            } else {
                return;
            };

            self.execute_commands(&room, &sender_username, &msg_body)
                .await;
        }
    }
}
