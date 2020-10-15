use crate::error::BotError;
use combine::parser::char::{digit, letter, spaces};
use combine::{many1, Parser};
use thiserror::Error;

enum ParsedValue {
    Valid(i32),
    Invalid,
}

#[derive(Error, Debug)]
pub enum VariableParsingError {
    #[error("invalid variable value, must be a number")]
    InvalidValue,

    #[error("unconsumed input")]
    UnconsumedInput,
}

pub fn parse_set_variable(input: &str) -> Result<(String, i32), BotError> {
    let name = many1(letter()).map(|value: String| value);

    let value = many1(digit()).map(|value: String| match value.parse::<i32>() {
        Ok(num) => ParsedValue::Valid(num),
        _ => ParsedValue::Invalid,
    });

    let mut parser = name.skip(spaces().silent()).and(value);
    let (result, rest) = parser.parse(input)?;

    if rest.len() == 0 {
        match result {
            (variable_name, ParsedValue::Valid(value)) => Ok((variable_name, value)),
            _ => Err(BotError::VariableParsingError(
                VariableParsingError::InvalidValue,
            )),
        }
    } else {
        Err(BotError::VariableParsingError(
            VariableParsingError::UnconsumedInput,
        ))
    }
}
