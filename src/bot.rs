use crate::commands::parse_command;
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
use serde::{self, Deserialize, Serialize};
use std::clone::Clone;
use std::ops::Sub;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use url::Url;

//TODO move the config structs and read_config into their own file.

/// The "matrix" section of the config, which gives home server, login information, and etc.
#[derive(Serialize, Deserialize, Debug)]
pub struct MatrixConfig {
    /// Your homeserver of choice, as an FQDN without scheme or path
    pub home_server: String,

    /// Username to login as. Only the localpart.
    pub username: String,

    /// Bot account password.
    pub password: String,
}

const DEFAULT_OLDEST_MESSAGE_AGE: u64 = 15 * 60;

/// The "bot" section of the config file, for bot settings.
#[derive(Serialize, Deserialize, Debug)]
pub struct BotConfig {
    /// How far back from current time should we process a message?
    oldest_message_age: Option<u64>,
}

impl BotConfig {
    pub fn new() -> BotConfig {
        BotConfig {
            oldest_message_age: Some(DEFAULT_OLDEST_MESSAGE_AGE),
        }
    }

    /// Determine the oldest allowable message age, in seconds. If the
    /// setting is defined, use that value. If it is not defined, fall
    /// back to DEFAULT_OLDEST_MESSAGE_AGE (15 minutes).
    pub fn oldest_message_age(&self) -> u64 {
        match self.oldest_message_age {
            Some(seconds) => seconds,
            None => DEFAULT_OLDEST_MESSAGE_AGE,
        }
    }
}

/// Represents the toml config file for the dicebot.
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub matrix: MatrixConfig,
    pub bot: Option<BotConfig>,
}

/// The DiceBot struct itself is the core of the program, essentially the entrypoint
/// to the bot.
pub struct DiceBot {
    /// A reference to the configuration read in on application start.
    config: Config,

    /// The matrix SDK client.
    client: Client,

    /// Current state of the dice bot. Held in an Arc because it
    /// accessed by the multi-threaded matrix SDK event handlers.
    state: Arc<DiceBotState>,
}

impl DiceBot {
    /// Create a new dicebot with the given Matrix configuration and
    /// client. The dice bot is iniitalized with a fresh state.
    pub fn new(config: Config, client: Client) -> Self {
        DiceBot {
            config: config,
            client: client,
            state: Arc::new(DiceBotState::new()),
        }
    }
}

/// Holds state of the dice bot, for anything requiring mutable
/// transitions. This is a simple mutable trait whose values represent
/// the current state of the dicebot. It provides mutable methods to
/// change state.
#[derive(Clone, Copy)]
pub struct DiceBotState {
    logged_skipped_old_message: bool,
}

impl DiceBotState {
    /// Create initial dice bot state.
    fn new() -> DiceBotState {
        DiceBotState {
            logged_skipped_old_message: false,
        }
    }

    /// Log and record that we have skipped some old messages. This
    /// method will log once, and then no-op from that point on.
    fn skipped_old_messages(mut self) {
        if !self.logged_skipped_old_message {
            info!("Skipped some messages because they are too old.");
        }

        self.logged_skipped_old_message = true;
    }
}

/// Figure out the allowed oldest message age, in seconds. This will
/// be the defined oldest message age in the bot config, if the bot
/// configuration and associated "oldest_message_age" setting are
/// defined. If the bot config or the message setting are not defined,
/// it will defualt to 15 minutes.
fn get_oldest_message_age(config: &Config) -> u64 {
    let none_cfg;
    let bot_cfg = match &config.bot {
        Some(cfg) => cfg,
        None => {
            none_cfg = BotConfig::new();
            &none_cfg
        }
    };

    bot_cfg.oldest_message_age()
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

            match self.client.join_room_by_id(&room.room_id).await {
                Err(e) => warn!("Could not join room: {}", e.to_string()),
                _ => (),
            }
        }
    }

    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        if let SyncRoom::Joined(room) = room {
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
                return;
            }

            //Ignore messages that are older than configured duration.
            if !check_message_age(event, get_oldest_message_age(&self.config)) {
                self.state.skipped_old_messages();
                return;
            }

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

            info!("{} executed: {}", sender_username, msg_body);

            //we clone here to hold the lock for as little time as possible.
            let room_id = room.read().await.room_id.clone();
            let result = self.client.room_send(&room_id, content, None).await;

            match result {
                Err(e) => error!("Error sending message: {}", e.to_string()),
                Ok(_) => (),
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum BotError {
    /// Sync token couldn't be found.
    #[error("the sync token could not be retrieved")]
    SyncTokenRequired,
}

/// Run the matrix dice bot until program terminated, or a panic occurs.
/// Originally adapted from the matrix-rust-sdk command bot example.
pub async fn run_bot(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let homeserver_url = &config.matrix.home_server;
    let username = &config.matrix.username;
    let password = &config.matrix.password;

    let mut cache_dir = dirs::cache_dir().expect("no cache directory found");
    cache_dir.push("matrix-dicebot");

    //If the local json store has not been created yet, we need to do a single initial sync.
    //It stores data under username's localpart.
    let should_sync = {
        let mut cache = cache_dir.clone();
        cache.push(username.clone());
        !cache.exists()
    };

    let store = JsonStore::open(&cache_dir)?;
    let client_config = ClientConfig::new().state_store(Box::new(store));

    let homeserver_url = Url::parse(homeserver_url).expect("Couldn't parse the homeserver URL");
    let mut client = Client::new_with_config(homeserver_url, client_config).unwrap();

    client
        .login(username, password, None, Some("matrix dice bot"))
        .await?;

    info!("Logged in as {}", username);

    if should_sync {
        info!("Performing initial sync");
        client.sync(SyncSettings::default()).await?;
    }

    //Attach event handler.
    info!("Listening for commands");
    info!(
        "Oldest allowable message time is {} seconds ago",
        get_oldest_message_age(&config)
    );

    client
        .add_event_emitter(Box::new(DiceBot::new(config, client.clone())))
        .await;

    let token = client
        .sync_token()
        .await
        .ok_or(BotError::SyncTokenRequired)?;
    let settings = SyncSettings::default().token(token);

    //this keeps state from the server streaming in to the dice bot via the EventEmitter trait
    client.sync_forever(settings, |_| async {}).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oldest_message_default_no_setting_test() {
        let cfg = Config {
            matrix: MatrixConfig {
                home_server: "".to_owned(),
                username: "".to_owned(),
                password: "".to_owned(),
            },
            bot: Some(BotConfig {
                oldest_message_age: None,
            }),
        };

        assert_eq!(15 * 60, get_oldest_message_age(&cfg));
    }

    #[test]
    fn oldest_message_default_no_bot_config_test() {
        let cfg = Config {
            matrix: MatrixConfig {
                home_server: "".to_owned(),
                username: "".to_owned(),
                password: "".to_owned(),
            },
            bot: None,
        };

        assert_eq!(15 * 60, get_oldest_message_age(&cfg));
    }
}
