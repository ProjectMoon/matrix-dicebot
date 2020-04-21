use nom::{
    bytes::complete::{tag, take_while},
    character::complete::digit1,
    character::is_alphabetic,
    complete, many0, named,
    sequence::tuple,
    switch, tag, take_while, tuple, IResult,
};

use crate::commands::{Command, RollCommand};
use crate::dice::parser::parse_element_expression;
use crate::parser::eat_whitespace;

// Parse a roll expression.
fn parse_roll(input: &str) -> IResult<&str, RollCommand> {
    let (input, _) = eat_whitespace(input)?;
    let (input, expression) = parse_element_expression(input)?;
    Ok((input, RollCommand(expression)))
}

// Potentially parse a command expression.  If we recognize the command, an error should be raised
// if the command is misparsed.  If we don't recognize the command, ignore it.
pub fn parse_command(original_input: &str) -> IResult<&str, Option<Box<dyn Command>>> {
    let (input, _) = eat_whitespace(original_input)?;
    named!(command(&str) -> (&str, &str), tuple!(complete!(tag!("!")), complete!(take_while!(char::is_alphabetic))));
    let (input, command) = match command(input) {
        // Strip the exclamation mark
        Ok((input, (_, result))) => (input, result),
        Err(_e) => return Ok((original_input, None)),
    };
    let (input, command) = match command {
        "r" | "roll" => {
            let (input, command) = parse_roll(input)?;
            let command: Box<dyn Command> = Box::new(command);
            (input, command)
        }
        // No recognized command, ignore this.
        _ => return Ok((original_input, None)),
    };
    Ok((input, Some(command)))
}
