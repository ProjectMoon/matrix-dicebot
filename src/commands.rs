use crate::cofd::dice::DicePool;
use crate::dice::ElementExpression;
use crate::error::BotError;
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

    fn execute(&self) -> Execution {
        let help = match &self.0 {
            Some(topic) => topic.message(),
            _ => "There is no help for this topic",
        };

        let plain = format!("Help: {}", help);
        let html = format!("<p><strong>Help:</strong> {}", help.replace("\n", "<br/>"));
        Execution { plain, html }
    }
}

/// Parse a command string into a dynamic command execution trait
/// object. Returns an error if a command was recognized but not
/// parsed correctly. Returns Ok(None) if no command was recognized.
pub fn parse_command(s: &str) -> Result<Option<Box<dyn Command>>, BotError> {
    // match parser::parse_command(s) {
    //     Ok(Some(command)) => match &command {
    //         //Any command, or text transformed into non-command is
    //         //sent upwards.
    //         ("", Some(_)) | (_, None) => Ok(command),

    //         //TODO replcae with nom all_consuming?
    //         //Any unconsumed input (whitespace should already be
    //         // stripped) is considered a parsing error.
    //         _ => Err(format!("{}: malformed expression", s)),
    //     },
    //     Err(err) => Err(err),
    // }
    parser::parse_command(s)
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
        assert!(parse_command("!roll 1d20asdlfkj   ").is_err());
    }

    #[test]
    fn roll_dice_pool_malformed_expression_test() {
        assert!(parse_command("!pool 8abc").is_err());
        assert!(parse_command("!pool 8abc    ").is_err());
    }

    #[test]
    fn pool_whitespace_test() {
        assert!(parse_command("!pool ns3:8   ")
            .map(|p| p.is_some())
            .expect("was error"));
        assert!(parse_command("   !pool ns3:8")
            .map(|p| p.is_some())
            .expect("was error"));
        assert!(parse_command("   !pool ns3:8   ")
            .map(|p| p.is_some())
            .expect("was error"));
    }

    #[test]
    fn help_whitespace_test() {
        assert!(parse_command("!help stuff   ")
            .map(|p| p.is_some())
            .expect("was error"));
        assert!(parse_command("   !help stuff")
            .map(|p| p.is_some())
            .expect("was error"));
        assert!(parse_command("   !help stuff   ")
            .map(|p| p.is_some())
            .expect("was error"));
    }

    #[test]
    fn roll_whitespace_test() {
        assert!(parse_command("!roll 1d4 + 5d6 -3   ")
            .map(|p| p.is_some())
            .expect("was error"));
        assert!(parse_command("!roll 1d4 + 5d6 -3   ")
            .map(|p| p.is_some())
            .expect("was error"));
        assert!(parse_command("   !roll 1d4 + 5d6 -3   ")
            .map(|p| p.is_some())
            .expect("was error"));
    }
}
