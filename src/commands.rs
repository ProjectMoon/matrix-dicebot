use crate::cofd::dice::{roll_pool, DicePool, DicePoolWithContext};
use crate::context::Context;
use crate::db::errors::DataError;
use crate::dice::ElementExpression;
use crate::error::BotError;
use crate::help::HelpTopic;
use crate::roll::Roll;
use async_trait::async_trait;
use thiserror::Error;

pub mod parser;

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

pub struct RollCommand(ElementExpression);

#[async_trait]
impl Command for RollCommand {
    fn name(&self) -> &'static str {
        "roll regular dice"
    }

    async fn execute(&self, _ctx: &Context) -> Execution {
        let roll = self.0.roll();
        let plain = format!("Dice: {}\nResult: {}", self.0, roll);
        let html = format!(
            "<p><strong>Dice:</strong> {}</p><p><strong>Result</strong>: {}</p>",
            self.0, roll
        );
        Execution { plain, html }
    }
}

pub struct PoolRollCommand(DicePool);

#[async_trait]
impl Command for PoolRollCommand {
    fn name(&self) -> &'static str {
        "roll dice pool"
    }

    async fn execute(&self, ctx: &Context) -> Execution {
        let pool_with_ctx = DicePoolWithContext(&self.0, ctx);
        let roll_result = roll_pool(&pool_with_ctx).await;

        let (plain, html) = match roll_result {
            Ok(rolled_pool) => {
                let plain = format!("Pool: {}\nResult: {}", rolled_pool, rolled_pool.roll);
                let html = format!(
                    "<p><strong>Pool:</strong> {}</p><p><strong>Result</strong>: {}</p>",
                    rolled_pool, rolled_pool.roll
                );
                (plain, html)
            }
            Err(e) => {
                let plain = format!("Error: {}", e);
                let html = format!("<p><strong>Error:</strong> {}</p>", e);
                (plain, html)
            }
        };

        Execution { plain, html }
    }
}

pub struct HelpCommand(Option<HelpTopic>);

#[async_trait]
impl Command for HelpCommand {
    fn name(&self) -> &'static str {
        "help information"
    }

    async fn execute(&self, _ctx: &Context) -> Execution {
        let help = match &self.0 {
            Some(topic) => topic.message(),
            _ => "There is no help for this topic",
        };

        let plain = format!("Help: {}", help);
        let html = format!("<p><strong>Help:</strong> {}", help.replace("\n", "<br/>"));
        Execution { plain, html }
    }
}

pub struct GetAllVariablesCommand;

#[async_trait]
impl Command for GetAllVariablesCommand {
    fn name(&self) -> &'static str {
        "get all variables"
    }

    async fn execute(&self, ctx: &Context) -> Execution {
        let value = match ctx
            .db
            .variables
            .get_user_variables(&ctx.room_id, &ctx.username)
            .await
        {
            Ok(variables) => {
                let mut variable_list = variables
                    .into_iter()
                    .map(|(name, value)| format!(" - {} = {}", name, value))
                    .collect::<Vec<_>>();

                variable_list.sort();
                variable_list.join("\n")
            }
            Err(e) => format!("error getting variables: {}", e),
        };

        let plain = format!("Variables:\n{}", value);
        let html = format!(
            "<p><strong>Variables:</strong><br/>{}",
            value.replace("\n", "<br/>")
        );
        Execution { plain, html }
    }
}

pub struct GetVariableCommand(String);

#[async_trait]
impl Command for GetVariableCommand {
    fn name(&self) -> &'static str {
        "retrieve variable value"
    }

    async fn execute(&self, ctx: &Context) -> Execution {
        let name = &self.0;
        let value = match ctx
            .db
            .variables
            .get_user_variable(&ctx.room_id, &ctx.username, name)
            .await
        {
            Ok(num) => format!("{} = {}", name, num),
            Err(DataError::KeyDoesNotExist(_)) => format!("{} is not set", name),
            Err(e) => format!("error getting {}: {}", name, e),
        };

        let plain = format!("Variable: {}", value);
        let html = format!("<p><strong>Variable:</strong> {}", value);
        Execution { plain, html }
    }
}

pub struct SetVariableCommand(String, i32);

#[async_trait]
impl Command for SetVariableCommand {
    fn name(&self) -> &'static str {
        "set variable value"
    }

    async fn execute(&self, ctx: &Context) -> Execution {
        let name = &self.0;
        let value = self.1;
        let result = ctx
            .db
            .variables
            .set_user_variable(&ctx.room_id, &ctx.username, name, value)
            .await;

        let content = match result {
            Ok(_) => format!("{} = {}", name, value),
            Err(e) => format!("error setting {}: {}", name, e),
        };

        let plain = format!("Set Variable: {}", content);
        let html = format!("<p><strong>Set Variable:</strong> {}", content);
        Execution { plain, html }
    }
}

pub struct DeleteVariableCommand(String);

#[async_trait]
impl Command for DeleteVariableCommand {
    fn name(&self) -> &'static str {
        "delete variable"
    }

    async fn execute(&self, ctx: &Context) -> Execution {
        let name = &self.0;
        let value = match ctx
            .db
            .variables
            .delete_user_variable(&ctx.room_id, &ctx.username, name)
            .await
        {
            Ok(()) => format!("{} now unset", name),
            Err(DataError::KeyDoesNotExist(_)) => format!("{} is not currently set", name),
            Err(e) => format!("error deleting {}: {}", name, e),
        };

        let plain = format!("Remove Variable: {}", value);
        let html = format!("<p><strong>Remove Variable:</strong> {}", value);
        Execution { plain, html }
    }
}

/// Parse a command string into a dynamic command execution trait
/// object. Returns an error if a command was recognized but not
/// parsed correctly. Returns Ok(None) if no command was recognized.
pub fn parse(s: &str) -> Result<Box<dyn Command>, BotError> {
    match parser::parse_command(s) {
        Ok(Some(command)) => Ok(command),
        Ok(None) => Err(BotError::CommandError(CommandError::IgnoredCommand)),
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
