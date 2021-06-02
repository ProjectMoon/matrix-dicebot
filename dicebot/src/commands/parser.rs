/**
 * In addition to the terms of the AGPL, portions of this file are
 * governed by the terms of the MIT license, from the original
 * axfive-matrix-dicebot project.
 */
use crate::commands::{
    basic_rolling::RollCommand,
    cofd::PoolRollCommand,
    cthulhu::{CthAdvanceRoll, CthRoll},
    management::{CheckCommand, LinkCommand, RegisterCommand, UnlinkCommand, UnregisterCommand},
    misc::HelpCommand,
    rooms::{ListRoomsCommand, SetRoomCommand},
    variables::{
        DeleteVariableCommand, GetAllVariablesCommand, GetVariableCommand, SetVariableCommand,
    },
    Command,
};
use crate::error::BotError;
use combine::parser::char::{char, letter, space};
use combine::{any, many1, optional, Parser};
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum CommandParsingError {
    #[error("unrecognized command: {0}")]
    UnrecognizedCommand(String),

    #[error("parser error: {0}")]
    InternalParseError(#[from] combine::error::StringStreamError),
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

/// Atempt to convert text input to a Boxed command type. Shortens
/// boilerplate.
macro_rules! convert_to {
    ($type:ident, $input: expr) => {
        $type::try_from($input).map(|cmd| Box::new(cmd) as Box<dyn Command>)
    };
}

/// Potentially parse a command expression. If we recognize the
/// command, an error should be raised if the command is misparsed. If
/// we don't recognize the command, return an error.
pub fn parse_command(input: &str) -> Result<Box<dyn Command>, BotError> {
    match split_command(input) {
        Ok((cmd, cmd_input)) => match cmd.to_lowercase().as_ref() {
            "variables" => convert_to!(GetAllVariablesCommand, cmd_input),
            "get" => convert_to!(GetVariableCommand, cmd_input),
            "set" => convert_to!(SetVariableCommand, cmd_input),
            "del" => convert_to!(DeleteVariableCommand, cmd_input),
            "r" | "roll" => convert_to!(RollCommand, cmd_input),
            "rp" | "pool" => convert_to!(PoolRollCommand, cmd_input),
            "chance" => PoolRollCommand::chance_die().map(|cmd| Box::new(cmd) as Box<dyn Command>),
            "cthroll" => convert_to!(CthRoll, cmd_input),
            "cthadv" | "ctharoll" => convert_to!(CthAdvanceRoll, cmd_input),
            "help" => convert_to!(HelpCommand, cmd_input),
            "register" => convert_to!(RegisterCommand, cmd_input),
            "link" => convert_to!(LinkCommand, cmd_input),
            "unlink" => convert_to!(UnlinkCommand, cmd_input),
            "check" => convert_to!(CheckCommand, cmd_input),
            "unregister" => convert_to!(UnregisterCommand, cmd_input),
            "rooms" => convert_to!(ListRoomsCommand, cmd_input),
            "room" => convert_to!(SetRoomCommand, cmd_input),
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
    fn newline_test() {
        assert!(parse_command("\n!roll 1d4").is_ok());
    }

    #[test]
    fn whitespace_and_newline_test() {
        assert!(parse_command("    \n!roll 1d4").is_ok());
    }

    #[test]
    fn newline_and_whitespace_test() {
        assert!(parse_command("\n    !cthroll 50").is_ok());
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
        parse_command("!pool ns3:8   ").expect("was error");
        parse_command("   !pool ns3:8").expect("was error");
        parse_command("   !pool ns3:8   ").expect("was error");
    }

    #[test]
    fn help_whitespace_test() {
        parse_command("!help stuff   ").expect("was error");
        parse_command("   !help stuff").expect("was error");
        parse_command("   !help stuff   ").expect("was error");
    }

    #[test]
    fn roll_whitespace_test() {
        parse_command("!roll 1d4 + 5d6 -3   ").expect("was error");
        parse_command("!roll 1d4 + 5d6 -3   ").expect("was error");
        parse_command("   !roll 1d4 + 5d6 -3   ").expect("was error");
    }

    #[test]
    fn case_insensitive_test() {
        parse_command("!CTHROLL 40").expect("command parsing is not case sensitive.");
    }
}
