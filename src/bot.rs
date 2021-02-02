use crate::commands::{execute_command, CommandResult, ExecutionError, ResponseExtractor};
use crate::config::*;
use crate::context::{Context, RoomContext};
use crate::db::Database;
use crate::error::BotError;
use crate::matrix;
use crate::state::DiceBotState;
use dirs;
use futures::stream::{self, StreamExt};
use log::info;
use matrix_sdk::{self, identifiers::RoomId, Client, ClientConfig, JoinedRoom, SyncSettings};
use std::clone::Clone;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use url::Url;

pub mod event_handlers;

/// How many commands can be in one message. If the amount is higher
/// than this, we reject execution.
const MAX_COMMANDS_PER_MESSAGE: usize = 50;

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

/// Handle responding to a single command being executed. Wil print
/// out the full result of that command.
async fn handle_single_result(
    client: &Client,
    cmd_result: &CommandResult,
    respond_to: &str,
    room_id: &RoomId,
) {
    let html = cmd_result.message_html(respond_to);
    matrix::send_message(client, room_id, &html).await;
}

/// Handle responding to multiple commands being executed. Will print
/// out how many commands succeeded and failed (if any failed).
async fn handle_multiple_results(
    client: &Client,
    results: &[(&str, CommandResult)],
    respond_to: &str,
    room_id: &RoomId,
) {
    let errors: Vec<(&str, &ExecutionError)> = results
        .into_iter()
        .filter_map(|(cmd, result)| match result {
            Err(e) => Some((*cmd, e)),
            _ => None,
        })
        .collect();

    let message = if errors.len() == 0 {
        format!("{}: Executed {} commands", respond_to, results.len())
    } else {
        let failures: String = errors
            .iter()
            .map(|&(cmd, err)| format!("<strong>{}:</strong> {}", cmd, err))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "{}: Executed {} commands ({} failed)\n\nFailures:\n{}",
            respond_to,
            results.len(),
            errors.len(),
            failures
        )
        .replace("\n", "<br/>")
    };

    matrix::send_message(client, room_id, &message).await;
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
        let room_name: &str = &room.display_name().await.ok().unwrap_or_default();
        let room_id = room.room_id().clone();

        let commands: Vec<&str> = msg_body
            .lines()
            .filter(|line| line.starts_with("!"))
            .collect();

        //Up to 50 commands allowed, otherwise we send back an error.
        let results: Vec<(&str, CommandResult)> = if commands.len() < MAX_COMMANDS_PER_MESSAGE {
            stream::iter(commands)
                .then(|command| async move {
                    let ctx = Context {
                        db: self.db.clone(),
                        matrix_client: &self.client,
                        room: RoomContext::new_with_name(&room, room_name),
                        username: &sender_username,
                        message_body: &command,
                    };

                    let cmd_result = execute_command(&ctx).await;
                    (command, cmd_result)
                })
                .collect()
                .await
        } else {
            vec![("", Err(ExecutionError(BotError::MessageTooLarge)))]
        };

        if results.len() >= 1 {
            if results.len() == 1 {
                handle_single_result(&self.client, &results[0].1, sender_username, &room_id).await;
            } else if results.len() > 1 {
                handle_multiple_results(&self.client, &results, sender_username, &room_id).await;
            }

            info!("[{}] {} executed: {}", room_name, sender_username, msg_body);
        }
    }
}
