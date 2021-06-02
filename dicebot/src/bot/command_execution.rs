use crate::context::{Context, RoomContext};
use crate::db::sqlite::Database;
use crate::error::BotError;
use crate::logic;
use crate::matrix;
use crate::{
    commands::{execute_command, ExecutionResult, ResponseExtractor},
    models::Account,
};
use futures::stream::{self, StreamExt};
use matrix_sdk::{
    self,
    identifiers::{EventId, RoomId},
    room::Joined,
    Client,
};
use std::clone::Clone;
use std::convert::TryFrom;

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

/// Map an account's active room value to an actual matrix room, if
/// the account has an active room. This only retrieves the
/// user-specified active room, and doesn't perform any further
/// filtering.
fn get_account_active_room(client: &Client, account: &Account) -> Result<Option<Joined>, BotError> {
    let active_room = account
        .registered_user()
        .and_then(|u| u.active_room.as_deref())
        .map(|room_id| RoomId::try_from(room_id))
        .transpose()?
        .and_then(|active_room_id| client.get_joined_room(&active_room_id));

    Ok(active_room)
}

/// Execute a single command in the list of commands. Can fail if the
/// Account value cannot be created/fetched from the database, or if
/// room display names cannot be calculated. Otherwise, the success or
/// error of command execution itself is returned.
async fn execute_single_command(
    command: &str,
    db: &Database,
    client: &Client,
    origin_room: &Joined,
    sender: &str,
) -> ExecutionResult {
    let origin_ctx = RoomContext::new(origin_room, sender).await?;
    let account = logic::get_account(db, sender).await?;
    let active_room = get_account_active_room(client, &account)?;

    // Active room is used in secure command-issuing rooms. In
    // "public" rooms, where other users are, treat origin as the
    // active room.
    let active_room = active_room
        .as_ref()
        .filter(|_| origin_ctx.secure)
        .unwrap_or(origin_room);

    let active_ctx = RoomContext::new(active_room, sender).await?;

    let ctx = Context {
        account,
        db: db.clone(),
        matrix_client: client.clone(),
        origin_room: origin_ctx,
        username: &sender,
        active_room: active_ctx,
        message_body: &command,
    };

    execute_command(&ctx).await
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
            let result = execute_single_command(command, db, client, room, sender).await;
            (command.to_owned(), result)
        })
        .collect()
        .await
}
