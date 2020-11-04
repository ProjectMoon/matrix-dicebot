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

    Ok(DiceRoll {
        amounts: amounts,
        modifier: modifier,
    })
}

pub fn parse_advancement_roll(input: &str) -> Result<AdvancementRoll, DiceParsingError> {
    let input = input.trim();
    let target: u32 = input.parse().map_err(|_| DiceParsingError::InvalidAmount)?;

    if target <= 100 {
        Ok(AdvancementRoll {
            existing_skill: target,
        })
    } else {
        Err(DiceParsingError::InvalidAmount)
    }
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
        assert_eq!(AdvancementRoll { existing_skill: 60 }, result.unwrap());
    }

    #[test]
    fn advancement_roll_rejects_big_numbers() {
        assert!(parse_advancement_roll("3000").is_err());
    }

    #[test]
    fn advancement_roll_rejects_invalid_input() {
        assert!(parse_advancement_roll("abc").is_err());
    }
}
