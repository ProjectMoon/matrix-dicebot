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

pub struct RollCommand(ElementExpression);

pub trait Command {
    fn execute(&self) -> Execution;
}

impl Command for RollCommand {
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

/// Parse a command string into a dynamic command execution trait object.
/// Returns an error if a command was recognized but not parsed correctly.  Returns None if no
/// command was recognized.
pub fn parse_command(s: &str) -> Result<Option<Box<dyn Command>>, String> {
    // Ignore trailing input, if any.
    match parser::parse_command(s) {
        Ok((_, result)) => Ok(result),
        Err(err) => Err(err.to_string()),
    }
}
