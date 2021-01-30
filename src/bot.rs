use crate::commands::{execute_command, CommandResult, ExecutionError, ResponseExtractor};
use crate::config::*;
use crate::context::{Context, RoomContext};
use crate::db::Database;
use crate::error::BotError;
use crate::state::DiceBotState;
use dirs;
use log::{error, info};
use matrix_sdk::Error as MatrixError;
use matrix_sdk::{
    self,
    events::{
        room::message::{MessageEventContent::Notice, NoticeMessageEventContent},
        AnyMessageEventContent::RoomMessage,
    },
    identifiers::RoomId,
    Client, ClientConfig, JoinedRoom, SyncSettings,
};
//use matrix_sdk_common_macros::async_trait;
use std::clone::Clone;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use url::Url;

pub mod event_handlers;

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
    //let store = JsonStore::open(&cache_dir)?;
    let client_config = ClientConfig::new().store_path(cache_dir);
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

/// Handle responding to a single command being executed. Wil print
/// out the full result of that command.
async fn handle_single_result(
    client: &Client,
    cmd_result: &CommandResult,
    respond_to: &str,
    room_id: &RoomId,
) {
    let plain = cmd_result.message_plain(respond_to);
    let html = cmd_result.message_html(respond_to);

    let response = RoomMessage(Notice(NoticeMessageEventContent::html(plain, html)));

    let result = client.room_send(&room_id, response, None).await;
    if let Err(e) = result {
        let message = extract_error_message(e);
        error!("Error sending message: {}", message);
    };
}

/// Handle responding to multiple commands being executed. Will print
/// out how many commands succeeded and failed (if any failed).
async fn handle_multiple_results(
    client: &Client,
    results: &[CommandResult],
    respond_to: &str,
    room_id: &RoomId,
) {
    // TODO we should also pass in the original command so we can
    // properly link errors to commands in output.
    let errors: Vec<&ExecutionError> = results.iter().filter_map(|r| r.as_ref().err()).collect();

    let message = if errors.len() == 0 {
        format!("{}: Executed {} commands", respond_to, results.len(),)
    } else {
        format!(
            "{}: Executed {} commands ({} failed)",
            respond_to,
            results.len(),
            errors.len()
        )
    };

    let response = RoomMessage(Notice(NoticeMessageEventContent::html(&message, &message)));

    let result = client.room_send(&room_id, response, None).await;
    if let Err(e) = result {
        let message = extract_error_message(e);
        error!("Error sending message: {}", message);
    };
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
        let client = self.client.clone();
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

    async fn execute_commands(&self, room: &JoinedRoom, sender_username: &str, msg_body: &str) {
        let room_name = room.display_name().await.ok().unwrap_or_default();
        let room_id = room.room_id().clone();

        let mut results = Vec::with_capacity(msg_body.lines().count());

        let commands = msg_body.trim().lines().filter(|line| line.starts_with("!"));

        for command in commands {
            let ctx = Context {
                db: self.db.clone(),
                matrix_client: &self.client,
                room: RoomContext::new_with_name(&room, &room_name),
                username: &sender_username,
                message_body: &command,
            };

            let cmd_result = execute_command(&ctx).await;
            results.push(cmd_result);
        }

        if results.len() >= 1 {
            if results.len() == 1 {
                handle_single_result(&self.client, &results[0], sender_username, &room_id).await;
            } else if results.len() > 1 {
                handle_multiple_results(&self.client, &results, sender_username, &room_id).await;
            }

            info!("[{}] {} executed: {}", room_name, sender_username, msg_body);
        }
    }
}
