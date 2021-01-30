use crate::context::Context;
use crate::error::BotError;
use async_trait::async_trait;
use thiserror::Error;

pub mod basic_rolling;
pub mod cofd;
pub mod cthulhu;
pub mod management;
pub mod misc;
pub mod parser;
pub mod variables;

/// A custom error type specifically related to parsing command text.
/// Does not wrap an execution failure.
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),

    #[error("ignored command")]
    IgnoredCommand,
}

/// A successfully executed command returns a message to be sent back
/// to the user in both plain text and HTML, one of which will be
/// displayed in the user's client depending on its capabilities.
#[derive(Debug)]
pub struct Execution {
    plain: String,
    html: String,
}

impl Execution {
    pub fn new(plain: String, html: String) -> CommandResult {
        Ok(Execution { plain, html })
    }

    /// Response message in plain text.
    pub fn plain(&self) -> String {
        self.plain.clone()
    }

    /// Response message in HTML.
    pub fn html(&self) -> String {
        self.html.clone()
    }
}

/// Wraps a command execution failure. Provides plain-text and HTML
/// formatting for any error message from the BotError type, similar
/// to how Response provides formatting for successfully executed
/// commands.
#[derive(Error, Debug)]
#[error("{0}")]
pub struct ExecutionError(#[from] BotError);

impl From<crate::db::errors::DataError> for ExecutionError {
    fn from(error: crate::db::errors::DataError) -> Self {
        Self(BotError::DataError(error))
    }
}

impl ExecutionError {
    /// Error message in plain text.
    pub fn plain(&self) -> String {
        format!("{}", self.0)
    }

    /// Error message in bolded HTML.
    pub fn html(&self) -> String {
        format!("<p><strong>{}</strong></p>", self.0)
    }
}

/// Wraps either a successful command execution response, or an error
/// that occurred.
pub type CommandResult = Result<Execution, ExecutionError>;

/// Extract response messages out of a type, whether it is success or
/// failure.
pub trait ResponseExtractor {
    /// Plain-text representation of the message, directly mentioning
    /// the username.
    fn message_plain(&self, username: &str) -> String;

    /// HTML representation of the message, directly mentioning the
    /// username.
    fn message_html(&self, username: &str) -> String;
}

impl ResponseExtractor for CommandResult {
    /// Error message in plain text.
    fn message_plain(&self, username: &str) -> String {
        match self {
            Ok(resp) => format!("{}\n{}", username, resp.plain()),
            Err(e) => format!("{}\n{}", username, e.plain()),
        }
    }

    /// Error message in bolded HTML.
    fn message_html(&self, username: &str) -> String {
        match self {
            Ok(resp) => format!("<p>{}</p>\n{}", username, resp.html),
            Err(e) => format!("<p>{}</p>\n{}", username, e.html()),
        }
    }
}

/// The trait that any command that can be executed must implement.
#[async_trait]
pub trait Command: Send + Sync {
    async fn execute(&self, ctx: &Context<'_>) -> CommandResult;
    fn name(&self) -> &'static str;
}

/// Attempt to execute a command, and return the content that should
/// go back to Matrix, if the command was executed (successfully or
/// not). If a command is determined to be ignored, this function will
/// return None, signifying that we should not send a response.
pub async fn execute_command(ctx: &Context<'_>) -> CommandResult {
    let cmd = parser::parse_command(&ctx.message_body)?;
    cmd.execute(ctx).await
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! dummy_room {
        () => {
            crate::context::RoomContext {
                id: &matrix_sdk::identifiers::room_id!("!fakeroomid:example.com"),
                display_name: "displayname",
            }
        };
    }

    #[tokio::test]
    async fn unrecognized_command() {
        let db = crate::db::Database::new_temp().unwrap();
        let ctx = Context {
            db: db,
            matrix_client: &matrix_sdk::Client::new("http://example.com").unwrap(),
            room: dummy_room!(),
            username: "myusername",
            message_body: "!notacommand",
        };
        let result = execute_command(&ctx).await;
        assert!(result.is_err());
    }
}
