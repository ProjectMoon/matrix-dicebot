use super::dice::{AdvancementRoll, DiceRoll, DiceRollModifier};
use crate::parser::dice::DiceParsingError;

//TOOD convert these to use parse_amounts from the common dice code.

fn parse_modifier(input: &str) -> Result<DiceRollModifier, DiceParsingError> {
    match input.trim() {
        "bb" => Ok(DiceRollModifier::TwoBonus),
        "b" => Ok(DiceRollModifier::OneBonus),
        "pp" => Ok(DiceRollModifier::TwoPenalty),
        "p" => Ok(DiceRollModifier::OnePenalty),
        "" => Ok(DiceRollModifier::Normal),
        _ => Err(DiceParsingError::InvalidModifiers),
    }
}

//Make diceroll take a vec of Amounts
//Split based on :, send first part to parse_modifier.
//Send second part to parse_amounts
pub fn parse_regular_roll(input: &str) -> Result<DiceRoll, DiceParsingError> {
    let (amount, modifiers_str) = crate::parser::dice::parse_single_amount(input)?;
    let modifier = parse_modifier(modifiers_str)?;
    Ok(DiceRoll { modifier, amount })
}

pub fn parse_advancement_roll(input: &str) -> Result<AdvancementRoll, DiceParsingError> {
    let input = input.trim();
    let (amounts, unconsumed_input) = crate::parser::dice::parse_single_amount(input)?;

    if unconsumed_input.len() == 0 {
        Ok(AdvancementRoll {
            existing_skill: amounts,
        })
    } else {
        Err(DiceParsingError::InvalidAmount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::dice::{Amount, DiceParsingError, Element, Operator};

    #[test]
    fn parse_modifier_rejects_bad_value() {
        let modifier = parse_modifier("qqq");
        assert!(matches!(modifier, Err(DiceParsingError::InvalidModifiers)))
    }

    #[test]
    fn parse_modifier_accepts_one_bonus() {
        let modifier = parse_modifier("b");
        assert!(matches!(modifier, Ok(DiceRollModifier::OneBonus)))
    }

    #[test]
    fn parse_modifier_accepts_two_bonus() {
        let modifier = parse_modifier("bb");
        assert!(matches!(modifier, Ok(DiceRollModifier::TwoBonus)))
    }

    #[test]
    fn parse_modifier_accepts_two_penalty() {
        let modifier = parse_modifier("pp");
        assert!(matches!(modifier, Ok(DiceRollModifier::TwoPenalty)))
    }

    #[test]
    fn parse_modifier_accepts_one_penalty() {
        let modifier = parse_modifier("p");
        assert!(matches!(modifier, Ok(DiceRollModifier::OnePenalty)))
    }

    #[test]
    fn parse_modifier_accepts_normal() {
        let modifier = parse_modifier("");
        assert!(matches!(modifier, Ok(DiceRollModifier::Normal)))
    }

    #[test]
    fn parse_modifier_accepts_normal_unaffected_by_whitespace() {
        let modifier = parse_modifier("         ");
        assert!(matches!(modifier, Ok(DiceRollModifier::Normal)))
    }

    #[test]
    fn regular_roll_accepts_single_number() {
        let result = parse_regular_roll("60");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                amount: Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                },
                modifier: DiceRollModifier::Normal
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_rejects_complex_expressions() {
        let result = parse_regular_roll("3 + abc + bob - 4");
        assert!(result.is_err());
    }

    #[test]
    fn regular_roll_accepts_two_bonus() {
        let result = parse_regular_roll("60 bb");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                amount: Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                },
                modifier: DiceRollModifier::TwoBonus
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_one_bonus() {
        let result = parse_regular_roll("60 b");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                amount: Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                },
                modifier: DiceRollModifier::OneBonus
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_two_penalty() {
        let result = parse_regular_roll("60 pp");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                amount: Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                },
                modifier: DiceRollModifier::TwoPenalty
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_one_penalty() {
        let result = parse_regular_roll("60 p");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                amount: Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                },
                modifier: DiceRollModifier::OnePenalty
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_whitespace() {
        assert!(parse_regular_roll("60     ").is_ok());
        assert!(parse_regular_roll("   60").is_ok());
        assert!(parse_regular_roll("   60    ").is_ok());

        assert!(parse_regular_roll("60bb     ").is_ok());
        assert!(parse_regular_roll("   60 bb").is_ok());
        assert!(parse_regular_roll("   60   bb    ").is_ok());

        assert!(parse_regular_roll("60b     ").is_ok());
        assert!(parse_regular_roll("   60 b").is_ok());
        assert!(parse_regular_roll("   60  b    ").is_ok());

        assert!(parse_regular_roll("60pp     ").is_ok());
        assert!(parse_regular_roll("   60 pp").is_ok());
        assert!(parse_regular_roll("   60 pp   ").is_ok());

        assert!(parse_regular_roll("60p     ").is_ok());
        assert!(parse_regular_roll("   60p  ").is_ok());
        assert!(parse_regular_roll("   60  p    ").is_ok());
    }

    #[test]
    fn advancement_roll_accepts_whitespacen() {
        assert!(parse_advancement_roll("60     ").is_ok());
        assert!(parse_advancement_roll("   60").is_ok());
        assert!(parse_advancement_roll("   60    ").is_ok());
    }

    #[test]
    fn advancement_roll_accepts_single_number() {
        let result = parse_advancement_roll("60");
        assert!(result.is_ok());
        assert_eq!(
            AdvancementRoll {
                existing_skill: Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                }
            },
            result.unwrap()
        );
    }

    #[test]
    fn advancement_roll_allows_big_numbers() {
        assert!(parse_advancement_roll("3000").is_ok());
    }

    #[test]
    fn advancement_roll_allows_variables() {
        let result = parse_advancement_roll("abc");
        assert!(result.is_ok());
        assert_eq!(
            AdvancementRoll {
                existing_skill: Amount {
                    operator: Operator::Plus,
                    element: Element::Variable(String::from("abc"))
                }
            },
            result.unwrap()
        );
    }

    #[test]
    fn advancement_roll_rejects_complex_expressions() {
        let result = parse_advancement_roll("3 + abc + bob - 4");
        assert!(result.is_err());
    }
}
