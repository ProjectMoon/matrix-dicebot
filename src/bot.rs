use crate::commands::execute_command;
use crate::config::*;
use crate::context::Context;
use crate::db::Database;
use crate::error::BotError;
use crate::state::DiceBotState;
use async_trait::async_trait;
use dirs;
use log::{debug, error, info, warn};
use matrix_sdk::Error as MatrixError;
use matrix_sdk::{
    self,
    events::{
        room::member::{MemberEventContent, MembershipState},
        room::message::{MessageEventContent, NoticeMessageEventContent, TextMessageEventContent},
        AnyMessageEventContent, StrippedStateEvent, SyncMessageEvent, SyncStateEvent,
    },
    Client, ClientConfig, EventEmitter, JsonStore, Room, SyncRoom, SyncSettings,
};
//use matrix_sdk_common_macros::async_trait;
use std::clone::Clone;
use std::ops::Sub;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use url::Url;

/// The DiceBot struct represents an active dice bot. The bot is not
/// connected to Matrix until its run() function is called.
pub struct DiceBot {
    /// A reference to the configuration read in on application start.
    config: Arc<Config>,

    /// The matrix client.
    client: Client,

    /// State of the dicebot
    state: Arc<RwLock<DiceBotState>>,

    /// Active database layer
    db: Database,
}

fn cache_dir() -> Result<PathBuf, BotError> {
    let mut dir = dirs::cache_dir().ok_or(BotError::NoCacheDirectoryError)?;
    dir.push("matrix-dicebot");
    Ok(dir)
}

/// Creates the matrix client.
fn create_client(config: &Config) -> Result<Client, BotError> {
    let cache_dir = cache_dir()?;
    let store = JsonStore::open(&cache_dir)?;
    let client_config = ClientConfig::new().state_store(Box::new(store));
    let homeserver_url = Url::parse(&config.matrix_homeserver())?;

    Ok(Client::new_with_config(homeserver_url, client_config)?)
}

/// Extracts more detailed error messages out of a matrix SDK error.
fn extract_error_message(error: MatrixError) -> String {
    use matrix_sdk::Error::RumaResponse;
    match error {
        RumaResponse(ruma_error) => ruma_error.to_string(),
        _ => error.to_string(),
    }
}

impl DiceBot {
    /// Create a new dicebot with the given configuration and state
    /// actor. This function returns a Result because it is possible
    /// for client creation to fail for some reason (e.g. invalid
    /// homeserver URL).
    pub fn new(
        config: &Arc<Config>,
        state: &Arc<RwLock<DiceBotState>>,
        db: &Database,
    ) -> Result<Self, BotError> {
        Ok(DiceBot {
            client: create_client(&config)?,
            config: config.clone(),
            state: state.clone(),
            db: db.clone(),
        })
    }

    /// Logs the bot into Matrix and listens for events until program
    /// terminated, or a panic occurs. Originally adapted from the
    /// matrix-rust-sdk command bot example.
    pub async fn run(self) -> Result<(), BotError> {
        let username = &self.config.matrix_username();
        let password = &self.config.matrix_password();

        //TODO provide a device id from config.
        let mut client = self.client.clone();
        client
            .login(username, password, None, Some("matrix dice bot"))
            .await?;

        info!("Logged in as {}", username);

        // Initial sync without event handler prevents responding to
        // messages received while bot was offline. TODO: selectively
        // respond to old messages? e.g. comands missed while offline.
        self.client.sync_once(SyncSettings::default()).await?;

        client.add_event_emitter(Box::new(self)).await;
        info!("Listening for commands");

        let token = client
            .sync_token()
            .await
            .ok_or(BotError::SyncTokenRequired)?;

        let settings = SyncSettings::default().token(token);

        // TODO replace with sync_with_callback for cleaner shutdown
        // process.
        client.sync(settings).await;
        Ok(())
    }

    async fn execute_commands(&self, room: &Room, sender_username: &str, msg_body: &str) {
        let room_name = room.display_name().clone();
        let room_id = room.room_id.clone();

        let mut results = Vec::with_capacity(msg_body.lines().count());

        for command in msg_body.lines() {
            if !command.is_empty() {
                let ctx = Context::new(&self.db, &room_id.as_str(), &sender_username, &command);
                if let Some(cmd_result) = execute_command(&ctx).await {
                    results.push(cmd_result);
                }
            }
        }

        if results.len() >= 1 {
            if results.len() == 1 {
                let cmd_result = &results[0];
                let response = AnyMessageEventContent::RoomMessage(MessageEventContent::Notice(
                    NoticeMessageEventContent::html(
                        cmd_result.plain.clone(),
                        cmd_result.html.clone(),
                    ),
                ));

                let result = self.client.room_send(&room_id, response, None).await;
                if let Err(e) = result {
                    let message = extract_error_message(e);
                    error!("Error sending message: {}", message);
                };
            } else if results.len() > 1 {
                let message = format!("{}: Executed {} commands", sender_username, results.len());
                let response = AnyMessageEventContent::RoomMessage(MessageEventContent::Notice(
                    NoticeMessageEventContent::html(&message, &message),
                ));

                let result = self.client.room_send(&room_id, response, None).await;
                if let Err(e) = result {
                    let message = extract_error_message(e);
                    error!("Error sending message: {}", message);
                };
            }

            info!("[{}] {} executed: {}", room_name, sender_username, msg_body);
        }
    }
}

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

async fn should_process<'a>(
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

/// This event emitter listens for messages with dice rolling commands.
/// Originally adapted from the matrix-rust-sdk examples.
#[async_trait]
impl EventEmitter for DiceBot {
    async fn on_room_member(
        &self,
        room: SyncRoom,
        room_member: &SyncStateEvent<MemberEventContent>,
    ) {
        //When joining a channel, we get join events from other users.
        //content is MemberContent, and it has a membership type.

        //Ignore if state_key is our username, because we only care about other users.
        let event_affects_us = if let Some(our_user_id) = self.client.user_id().await {
            room_member.state_key == our_user_id
        } else {
            false
        };

        let should_add = match room_member.content.membership {
            MembershipState::Join => true,
            MembershipState::Leave | MembershipState::Ban => false,
            _ => return,
        };

        //if event affects us and is leave/ban, delete all our info.
        //if event does not affect us, delete info only for that user.

        //TODO replace with call to new db.rooms thing.
        println!(
            "member {} recorded with action {:?} to/from db.",
            room_member.state_key, should_add
        );
    }

    async fn on_stripped_state_member(
        &self,
        room: SyncRoom,
        room_member: &StrippedStateEvent<MemberEventContent>,
        _: Option<MemberEventContent>,
    ) {
        if let SyncRoom::Invited(room) = room {
            if let Some(user_id) = self.client.user_id().await {
                if room_member.state_key != user_id {
                    return;
                }
            }

            let room = room.read().await;
            info!("Autojoining room {}", room.display_name());

            if let Err(e) = self.client.join_room_by_id(&room.room_id).await {
                warn!("Could not join room: {}", e.to_string())
            }
        }
    }

    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        if let SyncRoom::Joined(room) = room {
            let (msg_body, sender_username) =
                if let Ok((msg_body, sender_username)) = should_process(self, &event).await {
                    (msg_body, sender_username)
                } else {
                    return;
                };

            //we clone here to hold the lock for as little time as possible.
            let real_room = room.read().await.clone();

            self.execute_commands(&real_room, &sender_username, &msg_body)
                .await;
        }
    }
}
