/**
 * In addition to the terms of the AGPL, this file is governed by the
 * terms of the MIT license, from the original axfive-matrix-dicebot
 * project.
 */
use nom::bytes::complete::take_while;
use nom::error::ErrorKind as NomErrorKind;
use nom::Err as NomErr;
use nom::{
    alt, bytes::complete::tag, character::complete::digit1, complete, many0, named,
    sequence::tuple, tag, IResult,
};

use super::dice::*;

//******************************
//Legacy Code
//******************************

fn is_whitespace(input: char) -> bool {
    input == ' ' || input == '\n' || input == '\t' || input == '\r'
}

/// Eat whitespace, returning it
pub fn eat_whitespace(input: &str) -> IResult<&str, &str> {
    let (input, whitespace) = take_while(is_whitespace)(input)?;
    Ok((input, whitespace))
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Sign {
    Plus,
    Minus,
}

/// Intermediate parsed value for a keep-drop expression to indicate
/// which one it is.
enum ParsedKeepOrDrop<'a> {
    Keep(&'a str),
    Drop(&'a str),
    NotPresent,
}

macro_rules! too_big {
    ($input: expr) => {
        NomErr::Error(($input, NomErrorKind::TooLarge))
    };
}

/// Parse a dice expression.  Does not eat whitespace
fn parse_dice(input: &str) -> IResult<&str, Dice> {
    let (input, (count, _, sides)) = tuple((digit1, tag("d"), digit1))(input)?;
    let count: u32 = count.parse().map_err(|_| too_big!(count))?;
    let sides = sides.parse().map_err(|_| too_big!(sides))?;
    let (input, keep_drop) = parse_keep_or_drop(input, count)?;
    Ok((input, Dice::new(count, sides, keep_drop)))
}

/// Extract keep/drop number as a string. Fails if the value is not a
/// string.
fn parse_keep_or_drop_text<'a>(
    symbol: &'a str,
    input: &'a str,
) -> IResult<&'a str, ParsedKeepOrDrop<'a>> {
    let (parsed_kd, input) = match tuple::<&str, _, (_, _), _>((tag(symbol), digit1))(input) {
        // if ok, one of the expressions is present
        Ok((rest, (_, kd_expr))) => match symbol {
            "k" => (ParsedKeepOrDrop::Keep(kd_expr), rest),
            "dh" => (ParsedKeepOrDrop::Drop(kd_expr), rest),
            _ => panic!("Unrecogized keep-drop symbol: {}", symbol),
        },
        // otherwise absent (attempt to keep all dice)
        Err(_) => (ParsedKeepOrDrop::NotPresent, input),
    };

    Ok((input, parsed_kd))
}

/// Parse keep/drop expression, which consits of "k" or "dh" following
/// a dice expression. For example, "1d4h3" or "1d4dh2".
fn parse_keep_or_drop<'a>(input: &'a str, count: u32) -> IResult<&'a str, KeepOrDrop> {
    let (input, keep) = parse_keep_or_drop_text("k", input)?;
    let (input, drop) = parse_keep_or_drop_text("dh", input)?;

    use ParsedKeepOrDrop::*;
    let keep_drop: KeepOrDrop = match (keep, drop) {
        //Potential valid Keep expression.
        (Keep(keep), NotPresent) => match keep.parse().map_err(|_| too_big!(input))? {
            _i if _i > count || _i == 0 => Ok(KeepOrDrop::None),
            i => Ok(KeepOrDrop::Keep(i)),
        },
        //Potential valid Drop expression.
        (NotPresent, Drop(drop)) => match drop.parse().map_err(|_| too_big!(input))? {
            _i if _i >= count => Ok(KeepOrDrop::None),
            i => Ok(KeepOrDrop::Drop(i)),
        },
        //No Keep or Drop specified; regular behavior.
        (NotPresent, NotPresent) => Ok(KeepOrDrop::None),
        //Anything else is an error.
        _ => Err(NomErr::Error((input, NomErrorKind::Many1))),
    }?;

    Ok((input, keep_drop))
}

// Parse a single digit expression.  Does not eat whitespace
fn parse_bonus(input: &str) -> IResult<&str, u32> {
    let (input, bonus) = digit1(input)?;
    Ok((input, bonus.parse().unwrap()))
}

// Parse a sign expression.  Eats whitespace.
fn parse_sign(input: &str) -> IResult<&str, Sign> {
    let (input, _) = eat_whitespace(input)?;
    named!(sign(&str) -> Sign, alt!(
            complete!(tag!("+")) => { |_| Sign::Plus } |
            complete!(tag!("-")) => { |_| Sign::Minus }
    ));

    let (input, sign) = sign(input)?;
    Ok((input, sign))
}

// Parse an element expression.  Eats whitespace.
fn parse_element(input: &str) -> IResult<&str, Element> {
    let (input, _) = eat_whitespace(input)?;
    named!(element(&str) -> Element, alt!(
            parse_dice => { |d| Element::Dice(d) } |
            parse_bonus => { |b| Element::Bonus(b) }
    ));

    let (input, element) = element(input)?;
    Ok((input, element))
}

// Parse a signed element expression.  Eats whitespace.
fn parse_signed_element(input: &str) -> IResult<&str, SignedElement> {
    let (input, _) = eat_whitespace(input)?;
    let (input, sign) = parse_sign(input)?;
    let (input, _) = eat_whitespace(input)?;

    let (input, element) = parse_element(input)?;
    let element = match sign {
        Sign::Plus => SignedElement::Positive(element),
        Sign::Minus => SignedElement::Negative(element),
    };
    Ok((input, element))
}

// Parse a full element expression.  Eats whitespace.
pub fn parse_element_expression(input: &str) -> IResult<&str, ElementExpression> {
    named!(first_element(&str) -> SignedElement, alt!(
            parse_signed_element => { |e| e } |
            parse_element => { |e| SignedElement::Positive(e) }
    ));
    let (input, first) = first_element(input)?;
    let (input, rest) = if input.trim().is_empty() {
        (input, vec![first])
    } else {
        named!(rest_elements(&str) -> Vec<SignedElement>, many0!(parse_signed_element));
        let (input, mut rest) = rest_elements(input)?;
        rest.insert(0, first);
        (input, rest)
    };

    Ok((input, ElementExpression(rest)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dice_test() {
        assert_eq!(
            parse_dice("2d4"),
            Ok(("", Dice::new(2, 4, KeepOrDrop::None)))
        );
        assert_eq!(
            parse_dice("20d40"),
            Ok(("", Dice::new(20, 40, KeepOrDrop::None)))
        );
        assert_eq!(
            parse_dice("8d7"),
            Ok(("", Dice::new(8, 7, KeepOrDrop::None)))
        );
        assert_eq!(
            parse_dice("2d20k1"),
            Ok(("", Dice::new(2, 20, KeepOrDrop::Keep(1))))
        );
        assert_eq!(
            parse_dice("100d10k90"),
            Ok(("", Dice::new(100, 10, KeepOrDrop::Keep(90))))
        );
        assert_eq!(
            parse_dice("11d10k10"),
            Ok(("", Dice::new(11, 10, KeepOrDrop::Keep(10))))
        );
        assert_eq!(
            parse_dice("12d10k11"),
            Ok(("", Dice::new(12, 10, KeepOrDrop::Keep(11))))
        );
        assert_eq!(
            parse_dice("12d10k13"),
            Ok(("", Dice::new(12, 10, KeepOrDrop::None)))
        );
        assert_eq!(
            parse_dice("12d10k0"),
            Ok(("", Dice::new(12, 10, KeepOrDrop::None)))
        );
        assert_eq!(
            parse_dice("20d40dh5"),
            Ok(("", Dice::new(20, 40, KeepOrDrop::Drop(5))))
        );
        assert_eq!(
            parse_dice("8d7dh9"),
            Ok(("", Dice::new(8, 7, KeepOrDrop::None)))
        );
        assert_eq!(
            parse_dice("8d7dh8"),
            Ok(("", Dice::new(8, 7, KeepOrDrop::None)))
        );
    }

    #[test]
    fn cant_have_both_keep_and_drop_test() {
        let res = parse_dice("1d4k3dh2");
        assert!(res.is_err());
        match res {
            Err(NomErr::Error((_, kind))) => {
                assert_eq!(kind, NomErrorKind::Many1);
            }
            _ => panic!("Got success, expected error"),
        }
    }

    #[test]
    fn big_number_of_dice_doesnt_crash_test() {
        let res = parse_dice("64378631476346123874527551481376547657868536d4");
        assert!(res.is_err());
        match res {
            Err(NomErr::Error((input, kind))) => {
                assert_eq!(kind, NomErrorKind::TooLarge);
                assert_eq!(input, "64378631476346123874527551481376547657868536");
            }
            _ => panic!("Got success, expected error"),
        }
    }

    #[test]
    fn big_number_of_sides_doesnt_crash_test() {
        let res = parse_dice("1d423562312587425472658956278456298376234876");
        assert!(res.is_err());
        match res {
            Err(NomErr::Error((input, kind))) => {
                assert_eq!(kind, NomErrorKind::TooLarge);
                assert_eq!(input, "423562312587425472658956278456298376234876");
            }
            _ => panic!("Got success, expected error"),
        }
    }

    #[test]
    fn element_test() {
        assert_eq!(
            parse_element("  \t\n\r\n 8d7 \n"),
            Ok((" \n", Element::Dice(Dice::new(8, 7, KeepOrDrop::None))))
        );
        assert_eq!(
            parse_element("  \t\n\r\n 3d20k2 \n"),
            Ok((" \n", Element::Dice(Dice::new(3, 20, KeepOrDrop::Keep(2)))))
        );
        assert_eq!(
            parse_element("  \t\n\r\n 8 \n"),
            Ok((" \n", Element::Bonus(8)))
        );
    }

    #[test]
    fn signed_element_test() {
        assert_eq!(
            parse_signed_element("+ 7"),
            Ok(("", SignedElement::Positive(Element::Bonus(7))))
        );
        assert_eq!(
            parse_signed_element("  \t\n\r\n- 8 \n"),
            Ok((" \n", SignedElement::Negative(Element::Bonus(8))))
        );
        assert_eq!(
            parse_signed_element("  \t\n\r\n- 8d4 \n"),
            Ok((
                " \n",
                SignedElement::Negative(Element::Dice(Dice::new(8, 4, KeepOrDrop::None)))
            ))
        );
        assert_eq!(
            parse_signed_element("  \t\n\r\n- 8d4k4 \n"),
            Ok((
                " \n",
                SignedElement::Negative(Element::Dice(Dice::new(8, 4, KeepOrDrop::Keep(4))))
            ))
        );
        assert_eq!(
            parse_signed_element("  \t\n\r\n+ 8d4 \n"),
            Ok((
                " \n",
                SignedElement::Positive(Element::Dice(Dice::new(8, 4, KeepOrDrop::None)))
            ))
        );
    }

    #[test]
    fn element_expression_test() {
        assert_eq!(
            parse_element_expression("8d4"),
            Ok((
                "",
                ElementExpression(vec![SignedElement::Positive(Element::Dice(Dice::new(
                    8,
                    4,
                    KeepOrDrop::None
                )))])
            ))
        );
        assert_eq!(
            parse_element_expression("\t2d20k1 + 5"),
            Ok((
                "",
                ElementExpression(vec![
                    SignedElement::Positive(Element::Dice(Dice::new(2, 20, KeepOrDrop::Keep(1)))),
                    SignedElement::Positive(Element::Bonus(5)),
                ])
            ))
        );
        assert_eq!(
            parse_element_expression(" -  8d4 \n "),
            Ok((
                " \n ",
                ElementExpression(vec![SignedElement::Negative(Element::Dice(Dice::new(
                    8,
                    4,
                    KeepOrDrop::None
                )))])
            ))
        );
        assert_eq!(
            parse_element_expression("\t3d4k2 + 7 - 5 - 6d12dh3 + 1d1 + 53 1d5 "),
            Ok((
                " 1d5 ",
                ElementExpression(vec![
                    SignedElement::Positive(Element::Dice(Dice::new(3, 4, KeepOrDrop::Keep(2)))),
                    SignedElement::Positive(Element::Bonus(7)),
                    SignedElement::Negative(Element::Bonus(5)),
                    SignedElement::Negative(Element::Dice(Dice::new(6, 12, KeepOrDrop::Drop(3)))),
                    SignedElement::Positive(Element::Dice(Dice::new(1, 1, KeepOrDrop::None))),
                    SignedElement::Positive(Element::Bonus(53)),
                ])
            ))
        );
    }
}
