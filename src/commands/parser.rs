use nom::{
    alt,
    bytes::complete::{tag, take_while},
    character::complete::digit1,
    complete, many0, named,
    sequence::tuple,
    tag, IResult,
};

use crate::parser::eat_whitespace;
use crate::commands::{Command, RollCommand};
use crate::dice::parser::parse_element_expression;

// Parse a roll expression.
fn parse_roll(input: &str) -> IResult<&str, Command> {
    named!(invocation(&str) -> &str, alt!(complete!(tag!("!r")) | complete!(tag!("!roll"))));
    let (input, _) = eat_whitespace(input)?;
    let (input, _) = invocation(input)?;
    let (input, _) = eat_whitespace(input)?;
    let (input, expression) = parse_element_expression(input)?;
    Ok((input, Command::Roll(RollCommand(expression))))
}

// Parse a command expression.
pub fn parse_command(input: &str) -> IResult<&str, Command> {
    // Add new commands to alt!
    named!(command(&str) -> Command, alt!(parse_roll));
    command(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dice_test() {
        assert_eq!(parse_dice("2d4"), Ok(("", Dice::new(2, 4))));
        assert_eq!(parse_dice("20d40"), Ok(("", Dice::new(20, 40))));
        assert_eq!(parse_dice("8d7"), Ok(("", Dice::new(8, 7))));
    }

    #[test]
    fn element_test() {
        assert_eq!(
            parse_element("  \t\n\r\n 8d7 \n"),
            Ok((" \n", Element::Dice(Dice::new(8, 7))))
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
                SignedElement::Negative(Element::Dice(Dice::new(8, 4)))
            ))
        );
        assert_eq!(
            parse_signed_element("  \t\n\r\n+ 8d4 \n"),
            Ok((
                " \n",
                SignedElement::Positive(Element::Dice(Dice::new(8, 4)))
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
                    8, 4
                )))])
            ))
        );
        assert_eq!(
            parse_element_expression(" -  8d4 \n "),
            Ok((
                " \n ",
                ElementExpression(vec![SignedElement::Negative(Element::Dice(Dice::new(
                    8, 4
                )))])
            ))
        );
        assert_eq!(
            parse_element_expression("\t3d4 + 7 - 5 - 6d12 + 1d1 + 53 1d5 "),
            Ok((
                " 1d5 ",
                ElementExpression(vec![
                    SignedElement::Positive(Element::Dice(Dice::new(3, 4))),
                    SignedElement::Positive(Element::Bonus(7)),
                    SignedElement::Negative(Element::Bonus(5)),
                    SignedElement::Negative(Element::Dice(Dice::new(6, 12))),
                    SignedElement::Positive(Element::Dice(Dice::new(1, 1))),
                    SignedElement::Positive(Element::Bonus(53)),
                ])
            ))
        );
    }
}

