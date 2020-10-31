use crate::cofd::dice::{DicePool, DicePoolModifiers, DicePoolQuality};
use crate::error::BotError;
use crate::parser::{parse_amounts, DiceParsingError};
use combine::parser::char::{digit, spaces, string};
use combine::{choice, count, many1, one_of, Parser};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParsedInfo {
    Quality(DicePoolQuality),
    ExceptionalOn(i32),
}

pub fn parse_modifiers(input: &str) -> Result<DicePoolModifiers, DiceParsingError> {
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
        Err(DiceParsingError::UnconsumedInput)
    }
}

fn convert_to_info(parsed: &Vec<ParsedInfo>) -> Result<DicePoolModifiers, DiceParsingError> {
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
    let amounts = parse_amounts(&amounts_str)?;
    Ok(DicePool::new(amounts, modifiers))
}

pub fn create_chance_die() -> Result<DicePool, BotError> {
    Ok(DicePool::chance_die())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(matches!(result, Err(DiceParsingError::UnconsumedInput)));
    }

    #[test]
    fn multiple_quality_failure_test() {
        let result = parse_modifiers("ne");
        assert!(result.is_err());
        assert!(matches!(result, Err(DiceParsingError::InvalidModifiers)));
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
        assert!(matches!(result, Err(DiceParsingError::UnconsumedInput)));
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
        use crate::parser::*;
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
