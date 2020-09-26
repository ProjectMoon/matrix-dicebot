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
use std::ops::Sub;
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

/// The "bot" section of the config file, for bot settings.
#[derive(Serialize, Deserialize, Debug)]
pub struct BotConfig {
    /// How far back from current time should we process a message?
    pub oldest_message_sec: u64,
}

/// Represents the toml config file for the dicebot.
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub matrix: MatrixConfig,
    pub bot: BotConfig,
}

/// The DiceBot struct itself is the core of the program, essentially the entrypoint
/// to the bot.
pub struct DiceBot {
    config: Config,
    client: Client,
}

impl DiceBot {
    /// Create a new dicebot with the given Matrix client.
    pub fn new(config: Config, client: Client) -> Self {
        DiceBot { config, client }
    }
}

fn check_message_age(
    event: &SyncMessageEvent<MessageEventContent>,
    oldest_message_sec: u64,
) -> bool {
    let sending_time = event.origin_server_ts;
    let now = SystemTime::now();
    let oldest_timestamp = now.sub(Duration::new(oldest_message_sec, 0));

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
            let (msg_body, sender_username, sending_time) = if let SyncMessageEvent {
                content: MessageEventContent::Text(TextMessageEventContent { body, .. }),
                sender,
                origin_server_ts,
                ..
            } = event
            {
                (
                    body.clone(),
                    format!("@{}:{}", sender.localpart(), sender.server_name()),
                    origin_server_ts,
                )
            } else {
                (String::new(), String::new(), &SystemTime::UNIX_EPOCH)
            };

            //Ignore messages that are older than configured duration.
            if !check_message_age(event, self.config.bot.oldest_message_sec) {
                return;
            }

            //Command parser can handle non-commands, but faster to
            //not parse them.
            if !msg_body.starts_with("!") {
                trace!("Ignoring non-command: {}", msg_body);
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
    client
        .add_event_emitter(Box::new(DiceBot::new(config, client.clone())))
        .await;

    let token = client
        .sync_token()
        .await
        .ok_or(BotError::SyncTokenRequired)?;
    let settings = SyncSettings::default().token(token);

    //this keeps state from the server streaming in to the dice bot via the EventEmitter trait
    info!("Listening for commands");
    client.sync_forever(settings, |_| async {}).await;

    Ok(())
}
