use combine::parser::char::{digit, letter, spaces};
use combine::{many, many1, one_of, Parser};
use thiserror::Error;

//******************************
//New hotness
//******************************
#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum DiceParsingError {
    #[error("invalid amount")]
    InvalidAmount,

    #[error("modifiers not specified properly")]
    InvalidModifiers,

    #[error("extraneous input detected")]
    UnconsumedInput,

    #[error("{0}")]
    InternalParseError(#[from] combine::error::StringStreamError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operator {
    Plus,
    Minus,
}

impl Operator {
    pub fn mult(&self) -> i32 {
        match self {
            Operator::Plus => 1,
            Operator::Minus => -1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Element {
    Variable(String),
    Number(i32),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Amount {
    pub operator: Operator,
    pub element: Element,
}

/// Parse an expression of numbers and/or variables into elements
/// coupled with operators, where an operator is "+" or "-", and an
/// element is either a number or variable name. The first element
/// should not have an operator, but every one after that should.
/// Accepts expressions like "8", "10 + variablename", "variablename -
/// 3", etc. This function is currently common to systems that don't
/// deal with XdY rolls. Support for that will be added later. Parsers
/// utilzing this function should layer their own checks on top of
/// this; perhaps they do not want more than one expression, or some
/// other rules.
pub fn parse_amounts(input: &str) -> Result<Vec<Amount>, DiceParsingError> {
    let input = input.trim();

    let plus_or_minus = one_of("+-".chars());
    let maybe_sign = plus_or_minus.map(|sign: char| match sign {
        '+' => Operator::Plus,
        '-' => Operator::Minus,
        _ => Operator::Plus,
    });

    //TODO make this a macro or something
    let first = many1(letter())
        .or(many1(digit()))
        .skip(spaces().silent()) //Consume any space after first amount
        .map(|value: String| match value.parse::<i32>() {
            Ok(num) => Amount {
                operator: Operator::Plus,
                element: Element::Number(num),
            },
            _ => Amount {
                operator: Operator::Plus,
                element: Element::Variable(value),
            },
        });

    let variable_or_number =
        many1(letter())
            .or(many1(digit()))
            .map(|value: String| match value.parse::<i32>() {
                Ok(num) => Element::Number(num),
                _ => Element::Variable(value),
            });

    let sign_and_word = maybe_sign
        .skip(spaces().silent())
        .and(variable_or_number)
        .skip(spaces().silent())
        .map(|parsed: (Operator, Element)| Amount {
            operator: parsed.0,
            element: parsed.1,
        });

    let rest = many(sign_and_word).map(|expr: Vec<_>| expr);

    let mut parser = first.and(rest);

    //Maps the found expression into a Vec of Amount instances,
    //tacking the first one on.
    type ParsedAmountExpr = (Amount, Vec<Amount>);
    let (results, rest) = parser
        .parse(input)
        .map(|mut results: (ParsedAmountExpr, &str)| {
            let mut amounts = vec![(results.0).0];
            amounts.append(&mut (results.0).1);
            (amounts, results.1)
        })?;

    if rest.len() == 0 {
        Ok(results)
    } else {
        Err(DiceParsingError::UnconsumedInput)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_single_number_amount_test() {
        let result = parse_amounts("1");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            vec![Amount {
                operator: Operator::Plus,
                element: Element::Number(1)
            }]
        );

        let result = parse_amounts("10");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            vec![Amount {
                operator: Operator::Plus,
                element: Element::Number(10)
            }]
        );
    }

    #[test]
    fn parse_single_variable_amount_test() {
        let result = parse_amounts("asdf");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            vec![Amount {
                operator: Operator::Plus,
                element: Element::Variable("asdf".to_string())
            }]
        );

        let result = parse_amounts("nosis");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            vec![Amount {
                operator: Operator::Plus,
                element: Element::Variable("nosis".to_string())
            }]
        );
    }

    #[test]
    fn parse_complex_amount_expression() {
        assert!(parse_amounts("1 + myvariable - 2").is_ok());
    }
}
