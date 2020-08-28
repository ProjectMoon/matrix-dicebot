use crate::cofd::dice::DicePool;
use crate::dice::ElementExpression;
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
    fn execute(&self) -> Execution;
    fn name(&self) -> &'static str;
}

pub struct RollCommand(ElementExpression);

impl Command for RollCommand {
    fn name(&self) -> &'static str {
        "roll regular dice"
    }

    fn execute(&self) -> Execution {
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

    fn execute(&self) -> Execution {
        let roll = self.0.roll();
        let plain = format!("Pool: {}\nResult: {}", self.0, roll);
        let html = format!(
            "<p><strong>Pool:</strong> {}</p><p><strong>Result</strong>: {}</p>",
            self.0, roll
        );
        Execution { plain, html }
    }
}

/// Parse a command string into a dynamic command execution trait object.
/// Returns an error if a command was recognized but not parsed correctly.  Returns None if no
/// command was recognized.
pub fn parse_command(s: &str) -> Result<Option<Box<dyn Command>>, String> {
    match parser::parse_command(s) {
        Ok((input, result)) => match (input, &result) {
            //This clause prevents bot from spamming messages to itself
            //after executing a previous command.
            ("", Some(_)) | (_, None) => Ok(result),
            _ => Err(format!("{}: malformed dice expression", s)),
        },
        Err(err) => Err(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chance_die_is_not_malformed() {
        assert!(parse_command("!chance").is_ok());
    }

    #[test]
    fn roll_malformed_expression_test() {
        assert!(parse_command("!roll 1d20asdlfkj").is_err());
    }

    #[test]
    fn roll_dice_pool_expression_test() {
        assert!(parse_command("!pool 8abc").is_err());
    }
}
