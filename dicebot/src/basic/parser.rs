/**
 * In addition to the terms of the AGPL, this file is governed by the
 * terms of the MIT license, from the original axfive-matrix-dicebot
 * project.
 */
use nom::bytes::complete::take_while;
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

/// Parse a dice expression.  Does not eat whitespace
fn parse_dice(input: &str) -> IResult<&str, Dice> {
    // parse main dice expression
    let (input, (count, _, sides)) = tuple((digit1, tag("d"), digit1))(input)?;

    // check for keep expression to keep highest dice (2d20k1)
    let (keep, input) = match tuple::<&str, _, (_, _), _>((tag("k"), digit1))(input) {
        // if ok, keep expression is present
        Ok(r) => (r.1.1, r.0),
        // otherwise absent and keep all dice
        Err(_) => (input, "")
    };

    // check for drop expression to drop highest dice (2d20dh1)
    let (drop, input) = match tuple::<&str, _, (_, _), _>((tag("dh"), digit1))(input) {
        // if ok, keep expression is present
        Ok(r) => (r.1.1, r.0),
        // otherwise absent and keep all dice
        Err(_) => (input, "")
    };

    let count: u32 = count.parse().unwrap();

    // don't allow keep greater than number of dice, and don't allow keep zero
    let mut keep: u32  = keep.parse().unwrap();
    if keep > count || keep == 0 {
        keep = count;
    }

    // don't allow drop greater than or equal to number of dice
    let mut drop: u32  = drop.parse().unwrap();
    if drop >= count {
        drop = 0;
    }

    Ok((
        input,
        Dice::new(count, sides.parse().unwrap(), keep, drop),
    ))
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
        assert_eq!(parse_dice("2d4"), Ok(("", Dice::new(2, 4, 2, 0))));
        assert_eq!(parse_dice("20d40"), Ok(("", Dice::new(20, 40, 20, 0))));
        assert_eq!(parse_dice("8d7"), Ok(("", Dice::new(8, 7, 8, 0))));
        assert_eq!(parse_dice("2d20k1"), Ok(("", Dice::new(2, 20, 1, 0))));
        assert_eq!(parse_dice("100d10k90"), Ok(("", Dice::new(100, 10, 90, 0))));
        assert_eq!(parse_dice("11d10k10"), Ok(("", Dice::new(11, 10, 10, 0))));
        assert_eq!(parse_dice("12d10k11"), Ok(("", Dice::new(12, 10, 11, 0))));
        assert_eq!(parse_dice("12d10k13"), Ok(("", Dice::new(12, 10, 12, 0))));
        assert_eq!(parse_dice("12d10k0"), Ok(("", Dice::new(12, 10, 12, 0))));
        assert_eq!(parse_dice("20d40d5"), Ok(("", Dice::new(20, 40, 20, 5))));
        assert_eq!(parse_dice("8d7d9"), Ok(("", Dice::new(8, 7, 8, 0))));
        assert_eq!(parse_dice("8d7d8"), Ok(("", Dice::new(8, 7, 8, 0))));
    }

    #[test]
    fn element_test() {
        assert_eq!(
            parse_element("  \t\n\r\n 8d7 \n"),
            Ok((" \n", Element::Dice(Dice::new(8, 7, 8, 0))))
        );
        assert_eq!(
            parse_element("  \t\n\r\n 3d20k2 \n"),
            Ok((" \n", Element::Dice(Dice::new(3, 20, 2, 0))))
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
                SignedElement::Negative(Element::Dice(Dice::new(8, 4, 8, 0)))
            ))
        );
        assert_eq!(
            parse_signed_element("  \t\n\r\n- 8d4k4 \n"),
            Ok((
                " \n",
                SignedElement::Negative(Element::Dice(Dice::new(8, 4, 4, 0)))
            ))
        );
        assert_eq!(
            parse_signed_element("  \t\n\r\n+ 8d4 \n"),
            Ok((
                " \n",
                SignedElement::Positive(Element::Dice(Dice::new(8, 4, 8, 0)))
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
                    8, 4, 8, 0
                )))])
            ))
        );
        assert_eq!(
            parse_element_expression("\t2d20k1 + 5"),
            Ok((
                "",
                ElementExpression(vec![
                    SignedElement::Positive(Element::Dice(Dice::new(2, 20, 1, 0))),
                    SignedElement::Positive(Element::Bonus(5)),
                ])
            ))
        );
        assert_eq!(
            parse_element_expression(" -  8d4 \n "),
            Ok((
                " \n ",
                ElementExpression(vec![SignedElement::Negative(Element::Dice(Dice::new(
                    8, 4, 8, 0
                )))])
            ))
        );
        assert_eq!(
            parse_element_expression("\t3d4k2 + 7 - 5 - 6d12d3 + 1d1 + 53 1d5 "),
            Ok((
                " 1d5 ",
                ElementExpression(vec![
                    SignedElement::Positive(Element::Dice(Dice::new(3, 4, 2, 0))),
                    SignedElement::Positive(Element::Bonus(7)),
                    SignedElement::Negative(Element::Bonus(5)),
                    SignedElement::Negative(Element::Dice(Dice::new(6, 12, 6, 3))),
                    SignedElement::Positive(Element::Dice(Dice::new(1, 1, 1, 0))),
                    SignedElement::Positive(Element::Bonus(53)),
                ])
            ))
        );
    }
}
