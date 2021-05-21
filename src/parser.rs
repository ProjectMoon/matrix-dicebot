use combine::error::ParseError;
use combine::parser::char::{digit, letter, spaces};
use combine::stream::Stream;
use combine::{many, many1, one_of, Parser};
use thiserror::Error;

/// Errors for dice parsing.
#[derive(Debug, Clone, PartialEq, Copy, Error)]
pub enum DiceParsingError {
    #[error("invalid amount")]
    InvalidAmount,

    #[error("modifiers not specified properly")]
    InvalidModifiers,

    #[error("extraneous input detected")]
    UnconsumedInput,

    #[error("{0}")]
    InternalParseError(#[from] combine::error::StringStreamError),

    #[error("number parsing error (too large?)")]
    ConversionError,

    #[error("unexpected element in expression")]
    WrongElementType,
}

impl From<std::num::ParseIntError> for DiceParsingError {
    fn from(_: std::num::ParseIntError) -> Self {
        DiceParsingError::ConversionError
    }
}

type ParseResult<T> = Result<T, DiceParsingError>;

/// A parsed operator for a number. Whether to add or remove it from
/// the total amount of dice rolled.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    Plus,
    Minus,
}

impl Operator {
    /// Calculate multiplier for how to convert the number. Returns 1
    /// for positive, and -1 for negative.
    pub fn mult(&self) -> i32 {
        match self {
            Operator::Plus => 1,
            Operator::Minus => -1,
        }
    }
}

/// One part of the dice amount in an expression. Can be a number or a
/// variable name.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Element {
    /// This element in the expression is a variable, which will be
    /// resolved to a number by consulting the dtaabase.
    Variable(String),

    /// This element is a simple number, and will be added or
    /// subtracted from the total dice amount depending on its
    /// corresponding Operator.
    Number(i32),
}

/// One part of the parsed dice rolling expression. Combines an
/// operator and an element into one struct. Examples of Amounts would
/// be "+4" or "- myvariable", which translate to Operator::Plus and
/// Element::Number(4), and Operator::Minus and
/// Element::Variable("myvariable"), respectively.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Amount {
    pub operator: Operator,
    pub element: Element,
}

/// Parser that attempt to convert the text at the start of the dice
/// parsing into an Amount instance.
fn first_amount_parser<Input>() -> impl Parser<Input, Output = ParseResult<Amount>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let map_first_amount = |value: String| {
        if value.chars().all(char::is_numeric) {
            let num = value.parse::<i32>()?;
            Ok(Amount {
                operator: Operator::Plus,
                element: Element::Number(num),
            })
        } else {
            Ok(Amount {
                operator: Operator::Plus,
                element: Element::Variable(value),
            })
        }
    };

    many1(letter())
        .or(many1(digit()))
        .skip(spaces().silent()) //Consume any space after first amount
        .map(map_first_amount)
}

/// Attempt to convert some text in the middle or end of the dice roll
/// string into an Amount.
fn amount_parser<Input>() -> impl Parser<Input, Output = ParseResult<Amount>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let plus_or_minus = one_of("+-".chars());
    let parse_operator = plus_or_minus.map(|sign: char| match sign {
        '+' => Operator::Plus,
        '-' => Operator::Minus,
        _ => Operator::Plus,
    });

    // Element must either be a proper i32, or a variable name.
    let map_element = |value: String| -> ParseResult<Element> {
        if value.chars().all(char::is_numeric) {
            let num = value.parse::<i32>()?;
            Ok(Element::Number(num))
        } else {
            Ok(Element::Variable(value))
        }
    };

    let parse_element = many1(letter()).or(many1(digit())).map(map_element);

    let element_parser = parse_operator
        .skip(spaces().silent())
        .and(parse_element)
        .skip(spaces().silent());

    let convert_to_amount = |(operator, element_result)| match element_result {
        Ok(element) => Ok(Amount { operator, element }),
        Err(e) => Err(e),
    };

    element_parser.map(convert_to_amount)
}

/// Parse an expression of numbers and/or variables into elements
/// coupled with operators, where an operator is "+" or "-", and an
/// element is either a number or variable name. The first element
/// should not have an operator, but every one after that should.
/// Accepts expressions like "8", "10 + variablename", "variablename -
/// 3", etc. This function is currently common to systems that don't
/// deal with XdY rolls. Support for that will be added later.
pub fn parse_amounts(input: &str) -> ParseResult<Vec<Amount>> {
    let input = input.trim();

    let remaining_amounts = many(amount_parser()).map(|amounts: Vec<ParseResult<Amount>>| amounts);
    let mut parser = first_amount_parser().and(remaining_amounts);

    // Collapses first amount + remaining amounts into a single Vec,
    // while collecting extraneous input.
    type ParsedAmountExpr = (ParseResult<Amount>, Vec<ParseResult<Amount>>);
    let (results, rest) = parser
        .parse(input)
        .map(|mut results: (ParsedAmountExpr, &str)| {
            let mut amounts = vec![(results.0).0];
            amounts.append(&mut (results.0).1);
            (amounts, results.1)
        })?;

    if rest.len() == 0 {
        // Any ParseResult errors will short-circuit the collect.
        results.into_iter().collect()
    } else {
        Err(DiceParsingError::UnconsumedInput)
    }
}

/// Parse an expression that expects a single number or variable. No
/// operators are allowed. This function is common to systems that
/// don't deal with XdY rolls. Currently. this function does not
/// support parsing negative numbers.
pub fn parse_single_amount(input: &str) -> ParseResult<Amount> {
    // TODO add support for negative numbers, as technically they
    // should be allowed.
    let input = input.trim();
    let mut parser = first_amount_parser().map(|amount: ParseResult<Amount>| amount);

    let (result, rest) = parser.parse(input)?;

    if rest.len() == 0 {
        result
    } else {
        Err(DiceParsingError::UnconsumedInput)
    }
}

#[cfg(test)]
mod parse_single_amount_tests {
    use super::*;

    #[test]
    fn parse_single_variable_test() {
        let result = parse_single_amount("abc");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Amount {
                operator: Operator::Plus,
                element: Element::Variable("abc".to_string())
            }
        )
    }

    // TODO add support for negative numbers in parse_single_amount
    // #[test]
    // fn parse_single_negative_number_test() {
    //     let result = parse_single_amount("-1");
    //     assert!(result.is_ok());
    //     assert_eq!(
    //         result.unwrap(),
    //         Amount {
    //             operator: Operator::Minus,
    //             element: Element::Number(1)
    //         }
    //     )
    // }

    #[test]
    fn parse_single_number_test() {
        let result = parse_single_amount("1");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Amount {
                operator: Operator::Plus,
                element: Element::Number(1)
            }
        )
    }

    #[test]
    fn parse_multiple_elements_test() {
        let result = parse_single_amount("1+abc");
        assert!(result.is_err());

        let result = parse_single_amount("abc+1");
        assert!(result.is_err());

        let result = parse_single_amount("-1-abc");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod parse_many_amounts_tests {
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
    fn parsing_huge_number_should_error() {
        // A number outside the bounds of i32 should not be a valid
        // parse.
        let result = parse_amounts("159875294375198734982379875392");
        assert!(result.is_err());
        assert!(result.unwrap_err() == DiceParsingError::ConversionError);
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
