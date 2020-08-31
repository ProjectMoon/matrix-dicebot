use crate::cofd::parser::{create_chance_die, parse_dice_pool};
use crate::commands::{Command, HelpCommand, PoolRollCommand, RollCommand};
use crate::dice::parser::parse_element_expression;
use crate::help::parse_help_topic;
use nom::bytes::streaming::tag;
use nom::error::ErrorKind as NomErrorKind;
use nom::Err as NomErr;
use nom::{character::complete::alpha1, IResult};

// Parse a roll expression.
fn parse_roll(input: &str) -> IResult<&str, Box<dyn Command>> {
    let (input, expression) = parse_element_expression(input)?;
    Ok((input, Box::new(RollCommand(expression))))
}

fn parse_pool_roll(input: &str) -> IResult<&str, Box<dyn Command>> {
    let (input, pool) = parse_dice_pool(input)?;
    Ok((input, Box::new(PoolRollCommand(pool))))
}

fn chance_die() -> IResult<&'static str, Box<dyn Command>> {
    let (input, pool) = create_chance_die()?;
    Ok((input, Box::new(PoolRollCommand(pool))))
}

fn help(topic: &str) -> IResult<&str, Box<dyn Command>> {
    let topic = parse_help_topic(topic);
    Ok(("", Box::new(HelpCommand(topic))))
}

/// Split an input string into its constituent command and "everything
/// else" parts. Extracts the command separately from its input (i.e.
/// rest of the line) and returns a tuple of (command_input, command).
/// Whitespace at the start and end of the command input is removed.
fn split_command(input: &str) -> IResult<&str, &str> {
    let input = input.trim_start();
    let (input, _) = tag("!")(input)?;

    let (mut command_input, command) = alpha1(input)?;
    command_input = command_input.trim();
    Ok((command_input, command))
}

/// Potentially parse a command expression. If we recognize the
/// command, an error should be raised if the command is misparsed. If
/// we don't recognize the command, ignore it and return None.
pub fn parse_command(input: &str) -> IResult<&str, Option<Box<dyn Command>>> {
    match split_command(input) {
        Ok((cmd_input, cmd)) => match cmd {
            "r" | "roll" => parse_roll(cmd_input).map(|(input, command)| (input, Some(command))),
            "rp" | "pool" => {
                parse_pool_roll(cmd_input).map(|(input, command)| (input, Some(command)))
            }
            "chance" => chance_die().map(|(input, command)| (input, Some(command))),
            "help" => help(cmd_input).map(|(input, command)| (input, Some(command))),
            // No recognized command, ignore this.
            _ => Ok((input, None)),
        },

        //TODO better way to do this?
        //If the input is not a command, or the message is incomplete
        //(empty), we declare this to be a non-command, and don't do
        //anything else with it.
        Err(NomErr::Error((_, NomErrorKind::Tag))) | Err(NomErr::Incomplete(_)) => Ok(("", None)),

        //All other errors passed up.
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_command_test() {
        let result = parse_command("not a command");
        assert!(result.is_ok());
        assert!(result.unwrap().1.is_none());
    }

    #[test]
    fn empty_message_test() {
        let result = parse_command("");
        assert!(result.is_ok());
        assert!(result.unwrap().1.is_none());
    }

    #[test]
    fn just_exclamation_point_test() {
        let result = parse_command("!");
        assert!(result.is_err());
    }

    #[test]
    fn basic_command_test() {
        assert_eq!(
            ("1d4", "roll"),
            split_command("!roll 1d4").expect("got parsing error")
        );
    }

    #[test]
    fn whitespace_at_start_test() {
        assert_eq!(
            ("1d4", "roll"),
            split_command("   !roll 1d4").expect("got parsing error")
        );
    }

    #[test]
    fn whitespace_at_end_test() {
        assert_eq!(
            ("1d4", "roll"),
            split_command("!roll 1d4   ").expect("got parsing error")
        );
    }

    #[test]
    fn whitespace_on_both_ends_test() {
        assert_eq!(
            ("1d4", "roll"),
            split_command("   !roll 1d4   ").expect("got parsing error")
        );
    }

    #[test]
    fn single_command_test() {
        assert_eq!(
            ("", "roll"),
            split_command("!roll").expect("got parsing error")
        );

        assert_eq!(
            ("", "thisdoesnotexist"),
            split_command("!thisdoesnotexist").expect("got parsing error")
        );
    }

    #[test]
    fn bad_command_test() {
        assert!(split_command("roll 1d4").is_err());
        assert!(split_command("roll").is_err());
    }
}
