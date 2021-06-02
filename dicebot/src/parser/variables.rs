use combine::parser::char::{char, digit, letter, spaces};
use combine::{many1, optional, Parser};
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

    #[error("parser error: {0}")]
    InternalParseError(#[from] combine::error::StringStreamError),
}

pub fn parse_set_variable(input: &str) -> Result<(String, i32), VariableParsingError> {
    let name = many1(letter()).map(|value: String| value);

    let maybe_minus = optional(char('-')).map(|value: Option<char>| match value {
        Some(minus_sign) => String::from(minus_sign),
        _ => "".to_owned(),
    });

    let value = maybe_minus
        .and(many1(digit()))
        .map(|value: (String, String)| {
            let number = format!("{}{}", value.0, value.1);
            match number.parse::<i32>() {
                Ok(num) => ParsedValue::Valid(num),
                _ => ParsedValue::Invalid,
            }
        });

    let mut parser = name.skip(spaces().silent()).and(value);
    let (result, rest) = parser.parse(input)?;

    if rest.len() == 0 {
        match result {
            (variable_name, ParsedValue::Valid(value)) => Ok((variable_name, value)),
            _ => Err(VariableParsingError::InvalidValue),
        }
    } else {
        Err(VariableParsingError::UnconsumedInput)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_postive_number() {
        let result = parse_set_variable("myvar 5");
        assert!(result.is_ok());
        assert_eq!(("myvar".to_string(), 5), result.unwrap());
    }

    #[test]
    fn parse_negative_number() {
        let result = parse_set_variable("myvar -5");
        assert!(result.is_ok());
        assert_eq!(("myvar".to_string(), -5), result.unwrap());
    }
}
