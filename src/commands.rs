use crate::cofd::dice::DicePool;
use crate::context::Context;
use crate::dice::ElementExpression;
use crate::error::{BotError, CommandError};
use crate::help::HelpTopic;
use crate::roll::Roll;

pub mod parser;

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

pub trait Command {
    fn execute(&self, ctx: &Context) -> Execution;
    fn name(&self) -> &'static str;
}

pub struct RollCommand(ElementExpression);

impl Command for RollCommand {
    fn name(&self) -> &'static str {
        "roll regular dice"
    }

    fn execute(&self, _ctx: &Context) -> Execution {
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

impl Command for PoolRollCommand {
    fn name(&self) -> &'static str {
        "roll dice pool"
    }

    fn execute(&self, _ctx: &Context) -> Execution {
        let roll_result = self.0.roll();

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

impl Command for HelpCommand {
    fn name(&self) -> &'static str {
        "help information"
    }

    fn execute(&self, _ctx: &Context) -> Execution {
        let help = match &self.0 {
            Some(topic) => topic.message(),
            _ => "There is no help for this topic",
        };

        let plain = format!("Help: {}", help);
        let html = format!("<p><strong>Help:</strong> {}", help.replace("\n", "<br/>"));
        Execution { plain, html }
    }
}

pub struct GetVariableCommand(String);

impl Command for GetVariableCommand {
    fn name(&self) -> &'static str {
        "retrieve variable value"
    }

    fn execute(&self, ctx: &Context) -> Execution {
        let name = &self.0;
        let value = match ctx.db.get_user_variable(ctx.room_id, ctx.username, name) {
            Ok(Some(num)) => format!("{} = {}", name, num),
            Ok(None) => format!("{} is not set", name),
            Err(e) => format!("error getting {}: {}", name, e),
        };

        let plain = format!("Variable: {}", value);
        let html = format!("<p><strong>Variable:</strong> {}", value);
        Execution { plain, html }
    }
}

pub struct SetVariableCommand(String, i32);

impl Command for SetVariableCommand {
    fn name(&self) -> &'static str {
        "set variable value"
    }

    fn execute(&self, ctx: &Context) -> Execution {
        let name = &self.0;
        let value = self.1;
        let value = match ctx
            .db
            .set_user_variable(ctx.room_id, ctx.username, name, value)
        {
            Ok(_) => format!("{} = {}", name, value),
            Err(e) => format!("error setting {}: {}", name, e),
        };

        let plain = format!("Set Variable: {}", value);
        let html = format!("<p><strong>Set Variable:</strong> {}", value);
        Execution { plain, html }
    }
}

pub struct DeleteVariableCommand(String);

impl Command for DeleteVariableCommand {
    fn name(&self) -> &'static str {
        "delete variable"
    }

    fn execute(&self, ctx: &Context) -> Execution {
        let name = &self.0;
        let value = match ctx.db.delete_user_variable(ctx.room_id, ctx.username, name) {
            Ok(()) => format!("{} now unset", name),
            Err(e) => format!("error deleting {}: {}", name, e),
        };

        let plain = format!("Remove Variable: {}", value);
        let html = format!("<p><strong>Remove Variable:</strong> {}", value);
        Execution { plain, html }
    }
}

impl dyn Command {
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
}

pub struct CommandResult {
    pub plain: String,
    pub html: String,
}

/// Attempt to execute a command, and return the content that should
/// go back to Matrix, if the command was executed (successfully or
/// not). If a command is determined to be ignored, this function will
/// return None, signifying that we should not send a response.
pub fn execute_command<'a>(ctx: &'a Context) -> Option<CommandResult> {
    let res = Command::parse(ctx.message_body).map(|cmd| {
        let execution = cmd.execute(ctx);
        (execution.plain().into(), execution.html().into())
    });

    let (plain, html) = match res {
        Ok(plain_and_html) => plain_and_html,
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
        assert!(Command::parse("!chance").is_ok());
    }

    #[test]
    fn roll_malformed_expression_test() {
        assert!(Command::parse("!roll 1d20asdlfkj").is_err());
        assert!(Command::parse("!roll 1d20asdlfkj   ").is_err());
    }

    #[test]
    fn roll_dice_pool_malformed_expression_test() {
        assert!(Command::parse("!pool 8abc").is_err());
        assert!(Command::parse("!pool 8abc    ").is_err());
    }

    #[test]
    fn pool_whitespace_test() {
        Command::parse("!pool ns3:8   ").expect("was error");
        Command::parse("   !pool ns3:8").expect("was error");
        Command::parse("   !pool ns3:8   ").expect("was error");
    }

    #[test]
    fn help_whitespace_test() {
        Command::parse("!help stuff   ").expect("was error");
        Command::parse("   !help stuff").expect("was error");
        Command::parse("   !help stuff   ").expect("was error");
    }

    #[test]
    fn roll_whitespace_test() {
        Command::parse("!roll 1d4 + 5d6 -3   ").expect("was error");
        Command::parse("!roll 1d4 + 5d6 -3   ").expect("was error");
        Command::parse("   !roll 1d4 + 5d6 -3   ").expect("was error");
    }
}
