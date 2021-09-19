use crate::context::Context;
use crate::error::BotError;
use async_trait::async_trait;
use log::{error, info};
use thiserror::Error;

pub mod basic_rolling;
pub mod cofd;
pub mod cthulhu;
pub mod management;
pub mod misc;
pub mod parser;
pub mod rooms;
pub mod variables;

/// A custom error type specifically related to parsing command text.
/// Does not wrap an execution failure.
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),

    #[error("command can only be executed from encrypted direct message")]
    InsecureExecution,

    #[error("ignored command")]
    IgnoredCommand,
}

/// A successfully executed command returns a message to be sent back
/// to the user in HTML (plain text used as a fallback by message
/// formatter).
#[derive(Debug)]
pub struct Execution {
    html: String,
}

impl Execution {
    pub fn success(html: String) -> ExecutionResult {
        Ok(Execution { html })
    }

    /// Response message in HTML.
    pub fn html(&self) -> String {
        self.html.clone()
    }
}

/// Wraps either a successful command execution response, or an error
/// that occurred.
pub type ExecutionResult = Result<Execution, BotError>;

/// Extract response messages out of a type, whether it is success or
/// failure.
pub trait ResponseExtractor {
    /// HTML representation of the message, directly mentioning the
    /// username.
    fn message_html(&self, username: &str) -> String;

    fn message_plain(&self, username: &str) -> String;
}

impl ResponseExtractor for ExecutionResult {
    /// Error message in bolded HTML.
    fn message_html(&self, username: &str) -> String {
        // TODO use user display name too (element seems to render this
        // without display name)
        let username = format!(
            "<a href=\"https://matrix.to/#/{}\">{}</a>",
            username, username
        );

        match self {
            Ok(resp) => format!("<p>{}</p>", resp.html).replace("\n", "<br/>"),
            Err(e) => format!("<p>{}: <strong>{}</strong></p>", username, e).replace("\n", "<br/>"),
        }
    }

    fn message_plain(&self, username: &str) -> String {
        let message = match self {
            Ok(resp) => format!("{}", resp.html),
            Err(e) => format!("{}", e),
        };

        format!(
            "{}:\n{}",
            username,
            html2text::from_read(message.as_bytes(), message.len())
        )
    }
}

/// The trait that any command that can be executed must implement.
#[async_trait]
pub trait Command: Send + Sync {
    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult;
    fn name(&self) -> &'static str;
    fn is_secure(&self) -> bool;
}

/// Determine if we are allowed to execute this command. Currently the
/// rules are that secure commands must be executed in secure rooms
/// (encrypted + direct), and anything else can be executed where
/// ever. Later, we can add stuff like admin/regular user power
/// separation, etc.
fn execution_allowed(cmd: &(impl Command + ?Sized), ctx: &Context<'_>) -> Result<(), CommandError> {
    match cmd {
        cmd if cmd.is_secure() && ctx.is_secure() => Ok(()),
        cmd if cmd.is_secure() && !ctx.is_secure() => Err(CommandError::InsecureExecution),
        _ => Ok(()),
    }
}

/// Attempt to execute a command, and return the content that should
/// go back to Matrix, if the command was executed, whether or not the
/// command was successful.
pub async fn execute_command(ctx: &Context<'_>) -> ExecutionResult {
    let cmd = parser::parse_command(&ctx.message_body)?;

    let result = match execution_allowed(cmd.as_ref(), ctx) {
        Ok(_) => cmd.execute(ctx).await,
        Err(e) => Err(e.into()),
    };

    log_command(cmd.as_ref(), ctx, &result);
    result
}

/// Log result of an executed command.
fn log_command(cmd: &(impl Command + ?Sized), ctx: &Context, result: &ExecutionResult) {
    use substring::Substring;
    let command = match cmd.is_secure() {
        true => cmd.name(),
        false => ctx.message_body,
    };

    let dots = match command.len() {
        _len if _len > 30 => "[...]",
        _ => "",
    };

    let command = command.substring(0, 30);

    match result {
        Ok(_) => {
            info!(
                "[{}] {} <{}{}> - success",
                ctx.origin_room.display_name, ctx.username, command, dots
            );
        }
        Err(e) => {
            error!(
                "[{}] {} <{}{}> - {}",
                ctx.origin_room.display_name, ctx.username, command, dots, e
            );
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use management::RegisterCommand;
    use url::Url;

    macro_rules! dummy_room {
        () => {
            crate::context::RoomContext {
                id: &matrix_sdk::ruma::identifiers::room_id!("!fakeroomid:example.com"),
                display_name: "displayname".to_owned(),
                secure: false,
            }
        };
    }

    macro_rules! secure_room {
        () => {
            crate::context::RoomContext {
                id: &matrix_sdk::ruma::identifiers::room_id!("!fakeroomid:example.com"),
                display_name: "displayname".to_owned(),
                secure: true,
            }
        };
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn secure_context_secure_command_allows_execution() {
        let db_path = tempfile::NamedTempFile::new_in(".").unwrap();
        let db = crate::db::sqlite::Database::new(db_path.path().to_str().unwrap())
            .await
            .unwrap();

        let homeserver = Url::parse("http://example.com").unwrap();

        let ctx = Context {
            account: crate::models::Account::default(),
            db: db,
            matrix_client: matrix_sdk::Client::new(homeserver).unwrap(),
            origin_room: secure_room!(),
            active_room: secure_room!(),
            username: "myusername",
            message_body: "!notacommand",
        };

        let cmd = RegisterCommand;
        assert_eq!(execution_allowed(&cmd, &ctx).is_ok(), true);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn secure_context_insecure_command_allows_execution() {
        let db_path = tempfile::NamedTempFile::new_in(".").unwrap();
        let db = crate::db::sqlite::Database::new(db_path.path().to_str().unwrap())
            .await
            .unwrap();

        let homeserver = Url::parse("http://example.com").unwrap();

        let ctx = Context {
            account: crate::models::Account::default(),
            db: db,
            matrix_client: matrix_sdk::Client::new(homeserver).unwrap(),
            origin_room: secure_room!(),
            active_room: secure_room!(),
            username: "myusername",
            message_body: "!notacommand",
        };

        let cmd = variables::GetVariableCommand("".to_owned());
        assert_eq!(execution_allowed(&cmd, &ctx).is_ok(), true);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn insecure_context_insecure_command_allows_execution() {
        let db_path = tempfile::NamedTempFile::new_in(".").unwrap();
        let db = crate::db::sqlite::Database::new(db_path.path().to_str().unwrap())
            .await
            .unwrap();

        let homeserver = Url::parse("http://example.com").unwrap();

        let ctx = Context {
            account: crate::models::Account::default(),
            db: db,
            matrix_client: matrix_sdk::Client::new(homeserver).unwrap(),
            origin_room: dummy_room!(),
            active_room: dummy_room!(),
            username: "myusername",
            message_body: "!notacommand",
        };

        let cmd = variables::GetVariableCommand("".to_owned());
        assert_eq!(execution_allowed(&cmd, &ctx).is_ok(), true);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn insecure_context_secure_command_denies_execution() {
        let db_path = tempfile::NamedTempFile::new_in(".").unwrap();
        let db = crate::db::sqlite::Database::new(db_path.path().to_str().unwrap())
            .await
            .unwrap();

        let homeserver = Url::parse("http://example.com").unwrap();

        let ctx = Context {
            account: crate::models::Account::default(),
            db: db,
            matrix_client: matrix_sdk::Client::new(homeserver).unwrap(),
            origin_room: dummy_room!(),
            active_room: dummy_room!(),
            username: "myusername",
            message_body: "!notacommand",
        };

        let cmd = RegisterCommand;
        assert_eq!(execution_allowed(&cmd, &ctx).is_err(), true);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn unrecognized_command() {
        let db_path = tempfile::NamedTempFile::new_in(".").unwrap();
        let db = crate::db::sqlite::Database::new(db_path.path().to_str().unwrap())
            .await
            .unwrap();

        let homeserver = Url::parse("http://example.com").unwrap();

        let ctx = Context {
            account: crate::models::Account::default(),
            db: db,
            matrix_client: matrix_sdk::Client::new(homeserver).unwrap(),
            origin_room: dummy_room!(),
            active_room: dummy_room!(),
            username: "myusername",
            message_body: "!notacommand",
        };

        let result = execute_command(&ctx).await;
        assert!(result.is_err());
    }
}
