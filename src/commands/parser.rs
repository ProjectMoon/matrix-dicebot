use nom::{complete, named, tag, take_while, tuple, IResult};

use crate::commands::{Command, RollCommand};
use crate::dice::parser::parse_element_expression;
use crate::parser::eat_whitespace;

// Parse a roll expression.
fn parse_roll(input: &str) -> IResult<&str, Box<dyn Command>> {
    let (input, _) = eat_whitespace(input)?;
    let (input, expression) = parse_element_expression(input)?;
    Ok((input, Box::new(RollCommand(expression))))
}

/// Potentially parse a command expression.  If we recognize the command, an error should be raised
/// if the command is misparsed.  If we don't recognize the command, ignore it and return none
pub fn parse_command(original_input: &str) -> IResult<&str, Option<Box<dyn Command>>> {
    let (input, _) = eat_whitespace(original_input)?;
    named!(command(&str) -> (&str, &str), tuple!(complete!(tag!("!")), complete!(take_while!(char::is_alphabetic))));
    let (input, command) = match command(input) {
        // Strip the exclamation mark
        Ok((input, (_, result))) => (input, result),
        Err(_e) => return Ok((original_input, None)),
    };
    match command {
        "r" | "roll" => parse_roll(input).map(|(input, command)| (input, Some(command))),
        // No recognized command, ignore this.
        _ => Ok((original_input, None)),
    }
}
