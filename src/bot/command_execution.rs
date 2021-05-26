use crate::commands::{execute_command, ExecutionResult, ResponseExtractor};
use crate::context::{Context, RoomContext};
use crate::db::sqlite::Database;
use crate::error::BotError;
use crate::logic;
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
    let plain = cmd_result.message_plain(respond_to);
    matrix::send_message(client, room.room_id(), (&html, &plain), Some(event_id)).await;
}

/// Format failure messages nicely in either HTML or plain text. If
/// plain is true, plain-text will be returned. Otherwise, formatted
/// HTML.
fn format_failures(
    errors: &[(&str, &BotError)],
    commands_executed: usize,
    respond_to: &str,
    plain: bool,
) -> String {
    let respond_to = match plain {
        true => respond_to.to_owned(),
        false => format!(
            "<a href=\"https://matrix.to/#/{}\">{}</a>",
            respond_to, respond_to
        ),
    };

    let failures: Vec<String> = errors
        .iter()
        .map(|&(cmd, err)| format!("<strong>{}:</strong> {}", cmd, err))
        .collect();

    let message = format!(
        "{}: Executed {} commands ({} failed)\n\nFailures:\n{}",
        respond_to,
        commands_executed,
        errors.len(),
        failures.join("\n")
    )
    .replace("\n", "<br/>");

    match plain {
        true => html2text::from_read(message.as_bytes(), message.len()),
        false => message,
    }
}

/// Handle responding to multiple commands being executed. Will print
/// out how many commands succeeded and failed (if any failed).
pub(super) async fn handle_multiple_results(
    client: &Client,
    results: &[(String, ExecutionResult)],
    respond_to: &str,
    room: &Joined,
) {
    let user_pill = format!(
        "<a href=\"https://matrix.to/#/{}\">{}</a>",
        respond_to, respond_to
    );

    let errors: Vec<(&str, &BotError)> = results
        .into_iter()
        .filter_map(|(cmd, result)| match result {
            Err(e) => Some((cmd.as_ref(), e)),
            _ => None,
        })
        .collect();

    let (message, plain) = if errors.len() == 0 {
        (
            format!("{}: Executed {} commands", user_pill, results.len()),
            format!("{}: Executed {} commands", respond_to, results.len()),
        )
    } else {
        (
            format_failures(&errors, results.len(), respond_to, false),
            format_failures(&errors, results.len(), respond_to, true),
        )
    };

    matrix::send_message(client, room.room_id(), (&message, &plain), None).await;
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
        account: logic::get_account(db, &sender).await?,
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
                Err(e) => (command.to_owned(), Err(e)),
                Ok(ctx) => {
                    let cmd_result = execute_command(&ctx).await;
                    (command.to_owned(), cmd_result)
                }
            }
        })
        .collect()
        .await
}
