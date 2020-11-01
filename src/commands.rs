use crate::context::Context;
use crate::error::{BotError, BotError::CommandParsingError};
use async_trait::async_trait;
use parser::CommandParsingError::UnrecognizedCommand;
use thiserror::Error;

pub mod basic_rolling;
pub mod cofd;
pub mod cthulhu;
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
    async fn execute(&self, ctx: &Context) -> Execution;
    fn name(&self) -> &'static str;
}

/// Parse a command string into a dynamic command execution trait
/// object. Returns an error if a command was recognized but not
/// parsed correctly. Returns IgnoredCommand error if no command was
/// recognized.
pub fn parse(s: &str) -> Result<Box<dyn Command>, BotError> {
    match parser::parse_command(s) {
        Ok(command) => Ok(command),
        Err(CommandParsingError(UnrecognizedCommand(_))) => {
            Err(CommandError::IgnoredCommand.into())
        }
        Err(e) => Err(e),
    }
}

pub struct CommandResult {
    pub plain: String,
    pub html: String,
}

/// Attempt to execute a command, and return the content that should
/// go back to Matrix, if the command was executed (successfully or
/// not). If a command is determined to be ignored, this function will
/// return None, signifying that we should not send a response.
pub async fn execute_command(ctx: &Context) -> Option<CommandResult> {
    let res = parse(&ctx.message_body);

    let (plain, html) = match res {
        Ok(cmd) => {
            let execution = cmd.execute(ctx).await;
            (execution.plain().into(), execution.html().into())
        }
        Err(BotError::CommandError(CommandError::IgnoredCommand)) => return None,
        Err(e) => {
            let message = format!("Error parsing command: {}", e);
            let html_message = format!("<p><strong>{}</strong></p>", message);
            (message, html_message)
        }
    };

    let plain = format!("{}\n{}", ctx.username, plain);
    let html = format!("<p>{}</p>\n{}", ctx.username, html);

    Some(CommandResult {
        plain: plain,
        html: html,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chance_die_is_not_malformed() {
        assert!(parse("!chance").is_ok());
    }

    #[test]
    fn roll_malformed_expression_test() {
        assert!(parse("!roll 1d20asdlfkj").is_err());
        assert!(parse("!roll 1d20asdlfkj   ").is_err());
    }

    #[test]
    fn roll_dice_pool_malformed_expression_test() {
        assert!(parse("!pool 8abc").is_err());
        assert!(parse("!pool 8abc    ").is_err());
    }

    #[test]
    fn pool_whitespace_test() {
        parse("!pool ns3:8   ").expect("was error");
        parse("   !pool ns3:8").expect("was error");
        parse("   !pool ns3:8   ").expect("was error");
    }

    #[test]
    fn help_whitespace_test() {
        parse("!help stuff   ").expect("was error");
        parse("   !help stuff").expect("was error");
        parse("   !help stuff   ").expect("was error");
    }

    #[test]
    fn roll_whitespace_test() {
        parse("!roll 1d4 + 5d6 -3   ").expect("was error");
        parse("!roll 1d4 + 5d6 -3   ").expect("was error");
        parse("   !roll 1d4 + 5d6 -3   ").expect("was error");
    }
}
