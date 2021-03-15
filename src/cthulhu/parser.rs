use super::dice::{AdvancementRoll, DiceRoll, DiceRollModifier};
use crate::parser::DiceParsingError;

//TOOD convert these to use parse_amounts from the common dice code.

fn parse_modifier(input: &str) -> Result<DiceRollModifier, DiceParsingError> {
    if input.ends_with("bb") {
        Ok(DiceRollModifier::TwoBonus)
    } else if input.ends_with("b") {
        Ok(DiceRollModifier::OneBonus)
    } else if input.ends_with("pp") {
        Ok(DiceRollModifier::TwoPenalty)
    } else if input.ends_with("p") {
        Ok(DiceRollModifier::OnePenalty)
    } else {
        Ok(DiceRollModifier::Normal)
    }
}

//Make diceroll take a vec of Amounts
//Split based on :, send first part to parse_modifier.
//Send second part to parse_amounts

pub fn parse_regular_roll(input: &str) -> Result<DiceRoll, DiceParsingError> {
    let input: Vec<&str> = input.trim().split(":").collect();

    let (modifiers_str, amounts_str) = match input[..] {
        [amounts] => Ok(("", amounts)),
        [modifiers, amounts] => Ok((modifiers, amounts)),
        _ => Err(DiceParsingError::UnconsumedInput),
    }?;

    let modifier = parse_modifier(modifiers_str)?;
    let amounts = crate::parser::parse_amounts(amounts_str)?;

    Ok(DiceRoll { modifier, amounts })
}

pub fn parse_advancement_roll(input: &str) -> Result<AdvancementRoll, DiceParsingError> {
    let input = input.trim();
    let amounts = crate::parser::parse_amounts(input)?;

    Ok(AdvancementRoll {
        existing_skill: amounts,
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::parser::{Amount, Element, Operator};

    #[test]
    fn regular_roll_accepts_single_number() {
        let result = parse_regular_roll("60");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                amounts: vec![Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                }],
                modifier: DiceRollModifier::Normal
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_two_bonus() {
        let result = parse_regular_roll("bb:60");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                amounts: vec![Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                }],
                modifier: DiceRollModifier::TwoBonus
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_one_bonus() {
        let result = parse_regular_roll("b:60");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                amounts: vec![Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                }],
                modifier: DiceRollModifier::OneBonus
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_two_penalty() {
        let result = parse_regular_roll("pp:60");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                amounts: vec![Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                }],
                modifier: DiceRollModifier::TwoPenalty
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_one_penalty() {
        let result = parse_regular_roll("p:60");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                amounts: vec![Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                }],
                modifier: DiceRollModifier::OnePenalty
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_whitespacen() {
        assert!(parse_regular_roll("60     ").is_ok());
        assert!(parse_regular_roll("   60").is_ok());
        assert!(parse_regular_roll("   60    ").is_ok());

        assert!(parse_regular_roll("bb:60     ").is_ok());
        assert!(parse_regular_roll("   bb:60").is_ok());
        assert!(parse_regular_roll("   bb:60    ").is_ok());

        assert!(parse_regular_roll("b:60     ").is_ok());
        assert!(parse_regular_roll("   b:60").is_ok());
        assert!(parse_regular_roll("   b:60    ").is_ok());

        assert!(parse_regular_roll("pp:60     ").is_ok());
        assert!(parse_regular_roll("   pp:60").is_ok());
        assert!(parse_regular_roll("   pp:60    ").is_ok());

        assert!(parse_regular_roll("p:60     ").is_ok());
        assert!(parse_regular_roll("   p:60").is_ok());
        assert!(parse_regular_roll("   p:60    ").is_ok());
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
                existing_skill: vec![Amount {
                    operator: Operator::Plus,
                    element: Element::Number(60)
                }]
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
                existing_skill: vec![Amount {
                    operator: Operator::Plus,
                    element: Element::Variable(String::from("abc"))
                }]
            },
            result.unwrap()
        );
    }

    #[test]
    fn advancement_roll_allows_complex_expressions() {
        let result = parse_advancement_roll("3 + abc + bob - 4");
        assert!(result.is_ok());
        assert_eq!(
            AdvancementRoll {
                existing_skill: vec![
                    Amount {
                        operator: Operator::Plus,
                        element: Element::Number(3)
                    },
                    Amount {
                        operator: Operator::Plus,
                        element: Element::Variable(String::from("abc"))
                    },
                    Amount {
                        operator: Operator::Plus,
                        element: Element::Variable(String::from("bob"))
                    },
                    Amount {
                        operator: Operator::Minus,
                        element: Element::Number(4)
                    }
                ]
            },
            result.unwrap()
        );
    }
}
