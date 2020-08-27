use nom::{
    alt, bytes::complete::tag, character::complete::digit1, complete, many0, named,
    sequence::tuple, tag, IResult,
};

use crate::cofd::dice::{DicePool, DicePoolQuality};
use crate::parser::eat_whitespace;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DicePoolElement {
    NumberOfDice(u32),
    SuccessesForExceptional(u32),
    DicePoolQuality(DicePoolQuality),
}

// Parse a single digit expression.  Does not eat whitespace
fn parse_digit(input: &str) -> IResult<&str, u32> {
    let (input, num) = digit1(input)?;
    Ok((input, num.parse().unwrap()))
}

fn parse_quality(input: &str) -> IResult<&str, DicePoolQuality> {
    let (input, _) = eat_whitespace(input)?;
    named!(quality(&str) -> DicePoolQuality, alt!(
        complete!(tag!("n")) => { |_| DicePoolQuality::NineAgain } |
        complete!(tag!("e")) => { |_| DicePoolQuality::EightAgain } |
        complete!(tag!("r")) => { |_| DicePoolQuality::Rote }
    ));

    let (input, dice_pool_quality) = quality(input)?;
    Ok((input, dice_pool_quality))
}

fn parse_exceptional_requirement(input: &str) -> IResult<&str, u32> {
    let (input, _) = eat_whitespace(input)?;
    let (input, (_, successes)) = tuple((tag("s"), digit1))(input)?;
    Ok((input, successes.parse().unwrap()))
}

// Parse a dice pool element expression.  Eats whitespace.
fn parse_dice_pool_element(input: &str) -> IResult<&str, DicePoolElement> {
    let (input, _) = eat_whitespace(input)?;
    named!(element(&str) -> DicePoolElement, alt!(
        parse_digit => { |num| DicePoolElement::NumberOfDice(num) } |
        parse_quality => { |qual| DicePoolElement::DicePoolQuality(qual) } |
        parse_exceptional_requirement => { |succ| DicePoolElement::SuccessesForExceptional(succ) }
    ));

    let (input, element) = element(input)?;
    Ok((input, element))
}

fn find_elements(elements: Vec<DicePoolElement>) -> (Option<u32>, DicePoolQuality, u32) {
    let mut found_quality: Option<DicePoolQuality> = None;
    let mut found_count: Option<u32> = None;
    let mut found_successes_required: Option<u32> = None;

    for element in elements {
        if found_quality.is_some() && found_count.is_some() && found_successes_required.is_some() {
            break;
        }

        match element {
            DicePoolElement::NumberOfDice(found) => {
                if found_count.is_none() {
                    found_count = Some(found);
                }
            }
            DicePoolElement::DicePoolQuality(found) => {
                if found_quality.is_none() {
                    found_quality = Some(found);
                }
            }
            DicePoolElement::SuccessesForExceptional(found) => {
                if found_successes_required.is_none() {
                    found_successes_required = Some(found);
                }
            }
        };
    }

    let quality: DicePoolQuality = found_quality.unwrap_or(DicePoolQuality::TenAgain);
    let successes_for_exceptional: u32 = found_successes_required.unwrap_or(5);
    (found_count, quality, successes_for_exceptional)
}

fn convert_to_dice_pool(input: &str, elements: Vec<DicePoolElement>) -> IResult<&str, DicePool> {
    let (count, quality, successes_for_exceptional) = find_elements(elements);

    if count.is_some() {
        Ok((
            input,
            DicePool::new(count.unwrap(), successes_for_exceptional, quality),
        ))
    } else {
        use nom::error::ErrorKind;
        use nom::Err;
        Err(Err::Error((input, ErrorKind::Alt)))
    }
}

pub fn parse_dice_pool(input: &str) -> IResult<&str, DicePool> {
    named!(first_element(&str) -> DicePoolElement, alt!(
            parse_dice_pool_element => { |e| e }
    ));
    let (input, first) = first_element(input)?;
    let (input, elements) = if input.trim().is_empty() {
        (input, vec![first])
    } else {
        named!(rest_elements(&str) -> Vec<DicePoolElement>, many0!(parse_dice_pool_element));
        let (input, mut rest) = rest_elements(input)?;
        rest.insert(0, first);
        (input, rest)
    };

    convert_to_dice_pool(input, elements)
}

pub fn create_chance_die() -> IResult<&'static str, DicePool> {
    Ok(("", DicePool::chance_die()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_digit_test() {
        use nom::error::ErrorKind;
        use nom::Err;
        assert_eq!(parse_digit("1"), Ok(("", 1)));
        assert_eq!(parse_digit("10"), Ok(("", 10)));
        assert_eq!(
            parse_digit("adsf"),
            Err(Err::Error(("adsf", ErrorKind::Digit)))
        );
    }

    #[test]
    fn quality_test() {
        use nom::error::ErrorKind;
        use nom::Err;
        assert_eq!(parse_quality("n"), Ok(("", DicePoolQuality::NineAgain)));
        assert_eq!(parse_quality("e"), Ok(("", DicePoolQuality::EightAgain)));
        assert_eq!(parse_quality("r"), Ok(("", DicePoolQuality::Rote)));
        assert_eq!(parse_quality("b"), Err(Err::Error(("b", ErrorKind::Alt))));
    }

    #[test]
    fn multiple_quality_test() {
        assert_eq!(parse_quality("ner"), Ok(("er", DicePoolQuality::NineAgain)));
    }

    #[test]
    fn exceptional_success_test() {
        use nom::error::ErrorKind;
        use nom::Err;
        assert_eq!(parse_exceptional_requirement("s3"), Ok(("", 3)));
        assert_eq!(parse_exceptional_requirement("s10"), Ok(("", 10)));
        assert_eq!(parse_exceptional_requirement("s20b"), Ok(("b", 20)));
        assert_eq!(
            parse_exceptional_requirement("sab10"),
            Err(Err::Error(("ab10", ErrorKind::Digit)))
        );
    }

    #[test]
    fn dice_pool_element_expression_test() {
        use nom::error::ErrorKind;
        use nom::Err;

        assert_eq!(
            parse_dice_pool_element("8"),
            Ok(("", DicePoolElement::NumberOfDice(8)))
        );

        assert_eq!(
            parse_dice_pool_element("n"),
            Ok((
                "",
                DicePoolElement::DicePoolQuality(DicePoolQuality::NineAgain)
            ))
        );

        assert_eq!(
            parse_dice_pool_element("s3"),
            Ok(("", DicePoolElement::SuccessesForExceptional(3)))
        );

        assert_eq!(
            parse_dice_pool_element("8ns3"),
            Ok(("ns3", DicePoolElement::NumberOfDice(8)))
        );

        assert_eq!(
            parse_dice_pool_element("totallynotvalid"),
            Err(Err::Error(("totallynotvalid", ErrorKind::Alt)))
        );
    }

    #[test]
    fn dice_pool_number_only_test() {
        assert_eq!(
            parse_dice_pool("8"),
            Ok(("", DicePool::new(8, 5, DicePoolQuality::TenAgain)))
        );
    }

    #[test]
    fn dice_pool_number_with_quality() {
        assert_eq!(
            parse_dice_pool("8n"),
            Ok(("", DicePool::new(8, 5, DicePoolQuality::NineAgain)))
        );
    }

    #[test]
    fn dice_pool_number_with_success_change() {
        assert_eq!(
            parse_dice_pool("8s3"),
            Ok(("", DicePool::new(8, 3, DicePoolQuality::TenAgain)))
        );
    }

    #[test]
    fn dice_pool_with_quality_and_success_change() {
        assert_eq!(
            parse_dice_pool("8rs3"),
            Ok(("", DicePool::new(8, 3, DicePoolQuality::Rote)))
        );
    }
}
