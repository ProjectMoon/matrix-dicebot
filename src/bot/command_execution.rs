use crate::commands::{execute_command, ExecutionError, ExecutionResult, ResponseExtractor};
use crate::context::{Context, RoomContext};
use crate::db::sqlite::Database;
use crate::error::BotError;
use crate::matrix;
use futures::stream::{self, StreamExt};
use matrix_sdk::{self, identifiers::EventId, room::Joined, Client};
use std::clone::Clone;

/// Handle responding to a single command being executed. Wil print
/// out the full result of that command.
pub(super) async fn handle_single_result(
    client: &Client,
    cmd_result: &ExecutionResult,
    respond_to: &str,
    room: &Joined,
    event_id: EventId,
) {
    let html = cmd_result.message_html(respond_to);
    matrix::send_message(client, room.room_id(), &html, Some(event_id)).await;
}

/// Handle responding to multiple commands being executed. Will print
/// out how many commands succeeded and failed (if any failed).
pub(super) async fn handle_multiple_results(
    client: &Client,
    results: &[(String, ExecutionResult)],
    respond_to: &str,
    room: &Joined,
) {
    let respond_to = format!(
        "<a href=\"https://matrix.to/#/{}\">{}</a>",
        respond_to, respond_to
    );

    let errors: Vec<(&str, &ExecutionError)> = results
        .into_iter()
        .filter_map(|(cmd, result)| match result {
            Err(e) => Some((cmd.as_ref(), e)),
            _ => None,
        })
        .collect();

    let message = if errors.len() == 0 {
        format!("{}: Executed {} commands", respond_to, results.len())
    } else {
        let failures: Vec<String> = errors
            .iter()
            .map(|&(cmd, err)| format!("<strong>{}:</strong> {}", cmd, err))
            .collect();

        format!(
            "{}: Executed {} commands ({} failed)\n\nFailures:\n{}",
            respond_to,
            results.len(),
            errors.len(),
            failures.join("\n")
        )
        .replace("\n", "<br/>")
    };

    matrix::send_message(client, room.room_id(), &message, None).await;
}

/// Create a context for command execution. Can fai if the room
/// context creation fails.
async fn create_context<'a>(
    db: &'a Database,
    client: &'a Client,
    room: &'a Joined,
    sender: &'a str,
    command: &'a str,
) -> Result<Context<'a>, BotError> {
    let room_ctx = RoomContext::new(room, sender).await?;
    Ok(Context {
        db: db.clone(),
        matrix_client: client,
        room: room_ctx,
        username: &sender,
        message_body: &command,
    })
}

/// Attempt to execute all commands sent to the bot in a message. This
/// asynchronously executes all commands given to it. A Vec of all
/// commands and their execution results are returned.
pub(super) async fn execute(
    commands: Vec<&str>,
    db: &Database,
    client: &Client,
    room: &Joined,
    sender: &str,
) -> Vec<(String, ExecutionResult)> {
    stream::iter(commands)
        .then(|command| async move {
            match create_context(db, client, room, sender, command).await {
                Err(e) => (command.to_owned(), Err(ExecutionError(e))),
                Ok(ctx) => {
                    let cmd_result = execute_command(&ctx).await;
                    (command.to_owned(), cmd_result)
                }
            }
        })
        .collect()
        .await
}
