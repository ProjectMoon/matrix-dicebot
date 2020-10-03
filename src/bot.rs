use crate::commands::parse_command;
use crate::config::*;
use crate::error::BotError;
use crate::state::{DiceBotState, LogSkippedOldMessages};
use actix::Addr;
use dirs;
use log::{debug, error, info, trace, warn};
use matrix_sdk::{
    self,
    events::{
        room::member::MemberEventContent,
        room::message::{MessageEventContent, NoticeMessageEventContent, TextMessageEventContent},
        AnyMessageEventContent, StrippedStateEvent, SyncMessageEvent,
    },
    Client, ClientConfig, EventEmitter, JsonStore, SyncRoom, SyncSettings,
};
use matrix_sdk_common_macros::async_trait;
use std::clone::Clone;
use std::ops::Sub;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use url::Url;

/// The DiceBot struct represents an active dice bot. The bot is not
/// connected to Matrix until its run() function is called.
pub struct DiceBot {
    /// A reference to the configuration read in on application start.
    config: Arc<Config>,

    /// The matrix client.
    client: Client,

    /// Current state of the dice bot. Actor ref to the core state
    /// actor.
    state: Addr<DiceBotState>,
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

impl DiceBot {
    /// Create a new dicebot with the given configuration and state
    /// actor. This function returns a Result because it is possible
    /// for client creation to fail for some reason (e.g. invalid
    /// homeserver URL).
    pub fn new(config: &Arc<Config>, state_actor: Addr<DiceBotState>) -> Result<Self, BotError> {
        Ok(DiceBot {
            client: create_client(&config)?,
            config: config.clone(),
            state: state_actor,
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

        //If the local json store has not been created yet, we need to do a single initial sync.
        //It stores data under username's localpart.
        let should_sync = {
            let mut cache = cache_dir()?;
            cache.push(username);
            !cache.exists()
        };

        if should_sync {
            info!("Performing initial sync");
            self.client.sync(SyncSettings::default()).await?;
        }

        //Attach event handler.
        client.add_event_emitter(Box::new(self)).await;
        info!("Listening for commands");

        let token = client
            .sync_token()
            .await
            .ok_or(BotError::SyncTokenRequired)?;

        let settings = SyncSettings::default().token(token);

        //this keeps state from the server streaming in to the dice bot via the EventEmitter trait
        //TODO somehow figure out how to "sync_until" instead of sync_forever... copy code and modify?
        client.sync_forever(settings, |_| async {}).await;
        Ok(())
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

async fn should_process(
    bot: &DiceBot,
    event: &SyncMessageEvent<MessageEventContent>,
) -> Result<(String, String), BotError> {
    //Ignore messages that are older than configured duration.
    if !check_message_age(event, bot.config.oldest_message_age()) {
        let res = bot.state.send(LogSkippedOldMessages).await;

        if let Err(e) = res {
            error!("Actix error: {:?}", e);
        };

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

    //Command parser can handle non-commands, but faster to
    //not parse them.
    if !msg_body.starts_with("!") {
        trace!("Ignoring non-command: {}", msg_body);
        return Err(BotError::ShouldNotProcessError);
    }

    Ok((msg_body, sender_username))
}

/// This event emitter listens for messages with dice rolling commands.
/// Originally adapted from the matrix-rust-sdk examples.
#[async_trait]
impl EventEmitter for DiceBot {
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

            let (plain, html) = match parse_command(&msg_body) {
                Ok(Some(command)) => {
                    let command = command.execute();
                    (command.plain().into(), command.html().into())
                }
                Ok(None) => return, //Ignore non-commands.
                Err(e) => {
                    let message = format!("Error parsing command: {}", e);
                    let html_message = format!("<p><strong>{}</strong></p>", message);
                    (message, html_message)
                }
            };

            let plain = format!("{}\n{}", sender_username, plain);
            let html = format!("<p>{}</p>\n{}", sender_username, html);
            let content = AnyMessageEventContent::RoomMessage(MessageEventContent::Notice(
                NoticeMessageEventContent::html(plain, html),
            ));

            //we clone here to hold the lock for as little time as possible.
            let (room_name, room_id) = {
                let real_room = room.read().await;
                (real_room.display_name().clone(), real_room.room_id.clone())
            };

            let result = self.client.room_send(&room_id, content, None).await;
            if let Err(e) = result {
                error!("Error sending message: {}", e.to_string());
            };

            info!("[{}] {} executed: {}", room_name, sender_username, msg_body);
        }
    }
}
