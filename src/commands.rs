use crate::context::Context;
use async_trait::async_trait;
use thiserror::Error;

pub mod basic_rolling;
pub mod cofd;
pub mod cthulhu;
pub mod management;
pub mod misc;
pub mod parser;
pub mod variables;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),

    #[error("ignored command")]
    IgnoredCommand,
}

pub struct Execution {
    plain: String,
    html: String,
}

impl Execution {
    pub fn plain(&self) -> &str {
        &self.plain
    }

    pub fn html(&self) -> &str {
        &self.html
    }
}

#[async_trait]
pub trait Command: Send + Sync {
    async fn execute(&self, ctx: &Context<'_>) -> Execution;
    fn name(&self) -> &'static str;
}

#[derive(Debug)]
pub struct CommandResult {
    pub plain: String,
    pub html: String,
}

/// Attempt to execute a command, and return the content that should
/// go back to Matrix, if the command was executed (successfully or
/// not). If a command is determined to be ignored, this function will
/// return None, signifying that we should not send a response.
pub async fn execute_command(ctx: &Context<'_>) -> CommandResult {
    let res = parser::parse_command(&ctx.message_body);

    let (plain, html) = match res {
        Ok(cmd) => {
            let execution = cmd.execute(ctx).await;
            (execution.plain().into(), execution.html().into())
        }
        Err(e) => {
            let message = format!("Error parsing command: {}", e);
            let html_message = format!("<p><strong>{}</strong></p>", message);
            (message, html_message)
        }
    };

    let plain = format!("{}\n{}", ctx.username, plain);
    let html = format!("<p>{}</p>\n{}", ctx.username, html);

    CommandResult {
        plain: plain,
        html: html,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unrecognized_command() {
        let db = crate::db::Database::new_temp().unwrap();
        let ctx = Context {
            db: db,
            matrix_client: &matrix_sdk::Client::new("http://example.com").unwrap(),
            room_id: "myroomid",
            username: "myusername",
            message_body: "!notacommand",
        };
        let result = execute_command(&ctx).await;
        assert!(result.plain.contains("Error"));
    }
}
