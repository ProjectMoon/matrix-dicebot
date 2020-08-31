use crate::cofd::parser::{create_chance_die, parse_dice_pool};
use crate::commands::{Command, HelpCommand, PoolRollCommand, RollCommand};
use crate::dice::parser::parse_element_expression;
use crate::help::parse_help_topic;
use crate::parser::{eat_whitespace, trim};
use nom::{bytes::complete::tag, character::complete::alpha1, IResult};

// Parse a roll expression.
fn parse_roll(input: &str) -> IResult<&str, Box<dyn Command>> {
    let (input, _) = eat_whitespace(input)?;
    let (input, expression) = parse_element_expression(input)?;
    Ok((input, Box::new(RollCommand(expression))))
}

fn parse_pool_roll(input: &str) -> IResult<&str, Box<dyn Command>> {
    let (input, _) = eat_whitespace(input)?;
    let (input, pool) = parse_dice_pool(input)?;
    Ok((input, Box::new(PoolRollCommand(pool))))
}

fn chance_die() -> IResult<&'static str, Box<dyn Command>> {
    let (input, pool) = create_chance_die()?;
    Ok((input, Box::new(PoolRollCommand(pool))))
}

fn help(topic: &str) -> IResult<&str, Box<dyn Command>> {
    let (topic, _) = eat_whitespace(topic)?;
    let topic = parse_help_topic(&trim(topic));
    Ok(("", Box::new(HelpCommand(topic))))
}

/// Split an input string into its constituent command and "everything
/// else" parts. Extracts the command separately from its input (i.e.
/// rest of the line) and returns a tuple of (command_input, command).
fn split_command(input: &str) -> IResult<&str, &str> {
    let (input, _) = eat_whitespace(input)?;
    let (input, _) = tag("!")(input)?;
    let (command_input, command) = alpha1(input)?;
    let (command_input, _) = eat_whitespace(command_input)?;
    //TODO strip witespace from end of command input
    Ok((command_input, command))
}

/// Potentially parse a command expression. If we recognize the
/// command, an error should be raised if the command is misparsed. If
/// we don't recognize the command, ignore it and return none
pub fn parse_command(input: &str) -> IResult<&str, Option<Box<dyn Command>>> {
    let (cmd_input, cmd) = split_command(input)?;

    match cmd {
        "r" | "roll" => parse_roll(cmd_input).map(|(input, command)| (input, Some(command))),
        "rp" | "pool" => parse_pool_roll(cmd_input).map(|(input, command)| (input, Some(command))),
        "chance" => chance_die().map(|(input, command)| (input, Some(command))),
        "help" => help(cmd_input).map(|(input, command)| (input, Some(command))),
        // No recognized command, ignore this.
        _ => Ok((input, None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        //TODO currently it does not chop whitespace off the end.
        assert_eq!(
            ("1d4   ", "roll"),
            split_command("!roll 1d4   ").expect("got parsing error")
        );
    }

    #[test]
    fn whitespace_on_both_ends_test() {
        //TODO currently it does not chop whitespace off the end.
        assert_eq!(
            ("1d4   ", "roll"),
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
