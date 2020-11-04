use crate::basic::parser::parse_element_expression;
use crate::cofd::parser::{create_chance_die, parse_dice_pool};
use crate::commands::{
    basic_rolling::RollCommand,
    cofd::PoolRollCommand,
    cthulhu::{CthAdvanceRoll, CthRoll},
    misc::HelpCommand,
    variables::{
        DeleteVariableCommand, GetAllVariablesCommand, GetVariableCommand, SetVariableCommand,
    },
    Command,
};
use crate::cthulhu::parser::{parse_advancement_roll, parse_regular_roll};
use crate::error::BotError;
use crate::help::parse_help_topic;
use crate::variables::parse_set_variable;
use combine::parser::char::{char, letter, space};
use combine::{any, many1, optional, Parser};
use nom::Err as NomErr;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum CommandParsingError {
    #[error("unrecognized command: {0}")]
    UnrecognizedCommand(String),

    #[error("parser error: {0}")]
    InternalParseError(#[from] combine::error::StringStreamError),
}

// Parse a roll expression.
fn parse_roll(input: &str) -> Result<Box<dyn Command>, BotError> {
    let result = parse_element_expression(input);
    match result {
        Ok((rest, expression)) if rest.len() == 0 => Ok(Box::new(RollCommand(expression))),
        //Legacy code boundary translates nom errors into BotErrors.
        Ok(_) => Err(BotError::NomParserIncomplete),
        Err(NomErr::Error(e)) => Err(BotError::NomParserError(e.1)),
        Err(NomErr::Failure(e)) => Err(BotError::NomParserError(e.1)),
        Err(NomErr::Incomplete(_)) => Err(BotError::NomParserIncomplete),
    }
}

fn parse_get_variable_command(input: &str) -> Result<Box<dyn Command>, BotError> {
    Ok(Box::new(GetVariableCommand(input.to_owned())))
}

fn parse_set_variable_command(input: &str) -> Result<Box<dyn Command>, BotError> {
    let (variable_name, value) = parse_set_variable(input)?;
    Ok(Box::new(SetVariableCommand(variable_name, value)))
}

fn parse_delete_variable_command(input: &str) -> Result<Box<dyn Command>, BotError> {
    Ok(Box::new(DeleteVariableCommand(input.to_owned())))
}

fn parse_pool_roll(input: &str) -> Result<Box<dyn Command>, BotError> {
    let pool = parse_dice_pool(input)?;
    Ok(Box::new(PoolRollCommand(pool)))
}

fn parse_cth_roll(input: &str) -> Result<Box<dyn Command>, BotError> {
    let roll = parse_regular_roll(input)?;
    Ok(Box::new(CthRoll(roll)))
}

fn parse_cth_advancement_roll(input: &str) -> Result<Box<dyn Command>, BotError> {
    let roll = parse_advancement_roll(input)?;
    Ok(Box::new(CthAdvanceRoll(roll)))
}

fn chance_die() -> Result<Box<dyn Command>, BotError> {
    let pool = create_chance_die()?;
    Ok(Box::new(PoolRollCommand(pool)))
}

fn get_all_variables() -> Result<Box<dyn Command>, BotError> {
    Ok(Box::new(GetAllVariablesCommand))
}

fn help(topic: &str) -> Result<Box<dyn Command>, BotError> {
    let topic = parse_help_topic(topic);
    Ok(Box::new(HelpCommand(topic)))
}

/// Split an input string into its constituent command and "everything
/// else" parts. Extracts the command separately from its input (i.e.
/// rest of the line) and returns a tuple of (command_input, command).
/// Whitespace at the start and end of the command input is removed.
fn split_command(input: &str) -> Result<(String, String), CommandParsingError> {
    let input = input.trim();

    let exclamation = char('!');
    let word = many1(letter()).map(|value: String| value);
    let at_least_one_space = many1(space().silent()).map(|value: String| value);
    let cmd_input = optional(at_least_one_space.and(many1(any()).map(|value: String| value)));

    let mut parser = exclamation.and(word).and(cmd_input);

    //TODO make less wacky, possibly by mapping it into a struct and
    // making use of skip. This super-wacky tuple is:
    //  (parsed_input, rest)
    //Where parsed_input is:
    //  (!command, option<arguments>)
    //Where !command is:
    //  ('!', command)
    //Were option<arguments> is:
    // Option tuple of (whitespace, arguments)
    let (command, command_input) = match parser.parse(input)? {
        (((_, command), Some((_, command_input))), _) => (command, command_input),
        (((_, command), None), _) => (command, "".to_string()),
    };

    Ok((command, command_input))
}

/// Potentially parse a command expression. If we recognize the
/// command, an error should be raised if the command is misparsed. If
/// we don't recognize the command, ignore it and return None.
pub fn parse_command(input: &str) -> Result<Box<dyn Command>, BotError> {
    match split_command(input) {
        Ok((cmd, cmd_input)) => match cmd.as_ref() {
            "variables" => get_all_variables(),
            "get" => parse_get_variable_command(&cmd_input),
            "set" => parse_set_variable_command(&cmd_input),
            "del" => parse_delete_variable_command(&cmd_input),
            "r" | "roll" => parse_roll(&cmd_input),
            "rp" | "pool" => parse_pool_roll(&cmd_input),
            "cthroll" | "cthRoll" => parse_cth_roll(&cmd_input),
            "cthadv" | "cthARoll" => parse_cth_advancement_roll(&cmd_input),
            "chance" => chance_die(),
            "help" => help(&cmd_input),
            // No recognized command, ignore this.
            _ => Err(CommandParsingError::UnrecognizedCommand(cmd).into()),
        },
        //All other errors passed up.
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //TODO these errors don't seem to implement the right traits to do
    //eq checks or even unwrap_err!

    #[test]
    fn non_command_test() {
        let result = parse_command("not a command");
        assert!(result.is_err());
    }

    #[test]
    fn empty_message_test() {
        let result = parse_command("");
        assert!(result.is_err());
    }

    #[test]
    fn just_exclamation_mark_test() {
        let result = parse_command("!");
        assert!(result.is_err());
    }

    #[test]
    fn word_with_exclamation_mark_test() {
        let result1 = parse_command("hello !notacommand");
        assert!(result1.is_err());

        let result2 = parse_command("hello!");
        assert!(result2.is_err());

        let result3 = parse_command("hello!notacommand");
        assert!(result3.is_err());
    }

    #[test]
    fn basic_command_test() {
        assert_eq!(
            ("roll".to_string(), "1d4".to_string()),
            split_command("!roll 1d4").expect("got parsing error")
        );
    }

    #[test]
    fn whitespace_at_start_test() {
        assert_eq!(
            ("roll".to_string(), "1d4".to_string()),
            split_command("   !roll 1d4").expect("got parsing error")
        );
    }

    #[test]
    fn whitespace_at_end_test() {
        assert_eq!(
            ("roll".to_string(), "1d4".to_string()),
            split_command("!roll 1d4   ").expect("got parsing error")
        );
    }

    #[test]
    fn whitespace_on_both_ends_test() {
        assert_eq!(
            ("roll".to_string(), "1d4".to_string()),
            split_command("   !roll 1d4   ").expect("got parsing error")
        );
    }

    #[test]
    fn single_command_test() {
        assert_eq!(
            ("roll".to_string(), "".to_string()),
            split_command("!roll").expect("got parsing error")
        );

        assert_eq!(
            ("thisdoesnotexist".to_string(), "".to_string()),
            split_command("!thisdoesnotexist").expect("got parsing error")
        );
    }

    #[test]
    fn bad_command_test() {
        assert!(split_command("roll 1d4").is_err());
        assert!(split_command("roll").is_err());
    }
}
