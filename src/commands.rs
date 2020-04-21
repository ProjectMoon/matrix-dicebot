use crate::dice::ElementExpression;
use crate::roll::{Roll, Rolled};
use nom::error::ErrorKind;
use nom::IResult;
pub mod parser;

pub struct RollCommand(ElementExpression);

pub enum Command {
    Roll(RollCommand),
}

impl Command {
    pub fn parse<'a>(input: &'a str) -> IResult<&'a str, Command> {
        parser::parse_command(input)
    }

    // Type subject to change
    pub fn execute(self) -> String {
        match self {
            Command::Roll(command) => command.execute(),
        }
    }
}

impl RollCommand {
    pub fn execute(self) -> String {
        self.0.roll().to_string()
    }
}
