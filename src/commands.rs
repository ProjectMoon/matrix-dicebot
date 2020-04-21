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
            "<strong>Dice:</strong> {}<br><strong>Result</strong>: {}",
            self.0, roll
        );
        Execution { plain, html }
    }
}

pub fn parse_command(s: &str) -> Result<Option<Box<dyn Command>>, String> {
    // Ignore trailing input, if any.
    match parser::parse_command(s) {
        Ok((_, result)) => Ok(result),
        Err(err) => Err(err.to_string()),
    }
}
