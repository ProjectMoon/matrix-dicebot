use crate::cofd::dice::{Amount, DicePool, DicePoolModifiers, DicePoolQuality, Element, Operator};
use crate::error::BotError;
use combine::error::StringStreamError;
use combine::parser::char::{digit, letter, spaces, string};
use combine::{choice, count, many, many1, one_of, Parser};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParsedInfo {
    Quality(DicePoolQuality),
    ExceptionalOn(i32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiceParsingError {
    InvalidAmount,
    InvalidModifiers,
    UnconsumedInput,
}

impl std::fmt::Display for DiceParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::error::Error for DiceParsingError {
    fn description(&self) -> &str {
        self.as_str()
    }
}

impl DiceParsingError {
    fn as_str(&self) -> &str {
        use self::DiceParsingError::*;
        match *self {
            InvalidAmount => "invalid amount of dice",
            InvalidModifiers => "dice pool modifiers not specified properly",
            UnconsumedInput => "extraneous input detected",
        }
    }
}

pub fn parse_modifiers(input: &str) -> Result<DicePoolModifiers, BotError> {
    if input.len() == 0 {
        return Ok(DicePoolModifiers::default());
    }

    let input = input.trim();

    let quality = one_of("nerx".chars())
        .skip(spaces().silent())
        .map(|quality| match quality {
            'n' => ParsedInfo::Quality(DicePoolQuality::NineAgain),
            'e' => ParsedInfo::Quality(DicePoolQuality::EightAgain),
            'r' => ParsedInfo::Quality(DicePoolQuality::Rote),
            'x' => ParsedInfo::Quality(DicePoolQuality::NoExplode),
            _ => ParsedInfo::Quality(DicePoolQuality::TenAgain), //TODO add warning log
        });

    let exceptional_on = string("s")
        .and(many1(digit()))
        .map(|s| s.1) //Discard the s; only need the number
        .skip(spaces().silent())
        .map(|num_as_str: String| {
            ParsedInfo::ExceptionalOn(match num_as_str.parse::<i32>() {
                Ok(success_on) => success_on,
                Err(_) => 5, //TODO add warning log
            })
        });

    let mut parser = count(2, choice((quality, exceptional_on)))
        .skip(spaces().silent())
        .map(|modifiers: Vec<ParsedInfo>| modifiers);

    let (result, rest) = parser.parse(input)?;

    if rest.len() == 0 {
        convert_to_info(&result)
    } else {
        Err(BotError::DiceParsingError(
            DiceParsingError::UnconsumedInput,
        ))
    }
}

fn convert_to_info(parsed: &Vec<ParsedInfo>) -> Result<DicePoolModifiers, BotError> {
    use ParsedInfo::*;
    if parsed.len() == 0 {
        Ok(DicePoolModifiers::default())
    } else if parsed.len() == 1 {
        match parsed[0] {
            ExceptionalOn(exceptional_on) => {
                Ok(DicePoolModifiers::custom_exceptional_on(exceptional_on))
            }
            Quality(quality) => Ok(DicePoolModifiers::custom_quality(quality)),
        }
    } else if parsed.len() == 2 {
        match parsed[..] {
            [ExceptionalOn(exceptional_on), Quality(quality)] => {
                Ok(DicePoolModifiers::custom(quality, exceptional_on))
            }
            [Quality(quality), ExceptionalOn(exceptional_on)] => {
                Ok(DicePoolModifiers::custom(quality, exceptional_on))
            }
            _ => Err(DiceParsingError::InvalidModifiers.into()),
        }
    } else {
        //We don't expect this clause to be hit, because the parser works 0 to 2 times.
        Err(DiceParsingError::InvalidModifiers.into())
    }
}

/// Parse dice pool amounts into elements coupled with operators,
/// where an operator is "+" or "-", and an element is either a number
/// or variable name. The first element should not have an operator,
/// but every one after that should. Accepts expressions like "8", "10
/// + variablename", "variablename - 3", etc.
fn parse_pool_amount(input: &str) -> Result<Vec<Amount>, BotError> {
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
        Err(BotError::DiceParsingError(
            DiceParsingError::UnconsumedInput,
        ))
    }
}

pub fn parse_dice_pool(input: &str) -> Result<DicePool, BotError> {
    //The "modifiers:" part is optional. Assume amounts if no modifier
    //section found.
    let split = input.split(":").collect::<Vec<_>>();
    let (modifiers_str, amounts_str) = (match split[..] {
        [amounts] => Ok(("", amounts)),
        [modifiers, amounts] => Ok((modifiers, amounts)),
        _ => Err(BotError::DiceParsingError(
            DiceParsingError::UnconsumedInput,
        )),
    })?;

    let modifiers = parse_modifiers(modifiers_str)?;
    let amounts = parse_pool_amount(&amounts_str)?;
    Ok(DicePool::new(amounts, modifiers))
}

pub fn create_chance_die() -> Result<DicePool, StringStreamError> {
    Ok(DicePool::chance_die())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_number_amount_test() {
        let result = parse_pool_amount("1");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            vec![Amount {
                operator: Operator::Plus,
                element: Element::Number(1)
            }]
        );

        let result = parse_pool_amount("10");
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
        let result = parse_pool_amount("asdf");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            vec![Amount {
                operator: Operator::Plus,
                element: Element::Variable("asdf".to_string())
            }]
        );

        let result = parse_pool_amount("nosis");
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
        assert!(parse_pool_amount("1 + myvariable - 2").is_ok());
    }

    #[test]
    fn quality_test() {
        let result = parse_modifiers("n");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            DicePoolModifiers::custom_quality(DicePoolQuality::NineAgain)
        );

        let result = parse_modifiers("e");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            DicePoolModifiers::custom_quality(DicePoolQuality::EightAgain)
        );

        let result = parse_modifiers("r");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            DicePoolModifiers::custom_quality(DicePoolQuality::Rote)
        );

        let result = parse_modifiers("x");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            DicePoolModifiers::custom_quality(DicePoolQuality::NoExplode)
        );

        let result = parse_modifiers("b");
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(BotError::DiceParsingError(
                DiceParsingError::UnconsumedInput
            ))
        ));
    }

    #[test]
    fn multiple_quality_failure_test() {
        let result = parse_modifiers("ne");
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(BotError::DiceParsingError(
                DiceParsingError::InvalidModifiers
            ))
        ));
    }

    #[test]
    fn exceptional_success_test() {
        let result = parse_modifiers("s3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DicePoolModifiers::custom_exceptional_on(3));

        let result = parse_modifiers("s33");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            DicePoolModifiers::custom_exceptional_on(33)
        );

        let result = parse_modifiers("s3q");
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(BotError::DiceParsingError(
                DiceParsingError::UnconsumedInput
            ))
        ));
    }

    #[test]
    fn dice_pool_number_only_test() {
        let result = parse_dice_pool("8");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            DicePool::easy_pool(8, DicePoolQuality::TenAgain)
        );
    }

    #[test]
    fn dice_pool_number_with_quality() {
        let result = parse_dice_pool("n:8");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            DicePool::easy_pool(8, DicePoolQuality::NineAgain)
        );
    }

    #[test]
    fn dice_pool_number_with_success_change() {
        let modifiers = DicePoolModifiers::custom_exceptional_on(3);
        let result = parse_dice_pool("s3:8");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DicePool::easy_with_modifiers(8, modifiers));
    }

    #[test]
    fn dice_pool_with_quality_and_success_change() {
        let modifiers = DicePoolModifiers::custom(DicePoolQuality::Rote, 3);
        let result = parse_dice_pool("rs3:8");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DicePool::easy_with_modifiers(8, modifiers));
    }

    #[test]
    fn dice_pool_complex_expression_test() {
        let modifiers = DicePoolModifiers::custom(DicePoolQuality::Rote, 3);
        let amounts = vec![
            Amount {
                operator: Operator::Plus,
                element: Element::Number(8),
            },
            Amount {
                operator: Operator::Plus,
                element: Element::Number(10),
            },
            Amount {
                operator: Operator::Minus,
                element: Element::Number(2),
            },
            Amount {
                operator: Operator::Plus,
                element: Element::Variable("varname".to_owned()),
            },
        ];

        let expected = DicePool::new(amounts, modifiers);

        let result = parse_dice_pool("rs3:8+10-2+varname");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);

        let result = parse_dice_pool("rs3:8+10-   2 + varname");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);

        let result = parse_dice_pool("rs3  :  8+ 10 -2 + varname");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);

        //This one has tabs in it.
        let result = parse_dice_pool("  r	s3  :	8	+ 10 -2 + varname");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}
