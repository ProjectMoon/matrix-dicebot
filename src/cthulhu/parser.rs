use super::dice::{AdvancementRoll, DiceRoll, DiceRollModifier};
use crate::parser::DiceParsingError;

//TOOD convert these to use parse_amounts from the common dice code.

fn parse_modifier(input: &str) -> Result<(DiceRollModifier, &str), DiceParsingError> {
    if input.ends_with("bb") {
        Ok((DiceRollModifier::TwoBonus, input.trim_end_matches("bb")))
    } else if input.ends_with("b") {
        Ok((DiceRollModifier::OneBonus, input.trim_end_matches("b")))
    } else if input.ends_with("pp") {
        Ok((DiceRollModifier::TwoPenalty, input.trim_end_matches("pp")))
    } else if input.ends_with("p") {
        Ok((DiceRollModifier::OnePenalty, input.trim_end_matches("p")))
    } else {
        Ok((DiceRollModifier::Normal, input))
    }
}

pub fn parse_regular_roll(input: &str) -> Result<DiceRoll, DiceParsingError> {
    let input = input.trim();
    let (modifier, input) = parse_modifier(input)?;
    let target: u32 = input.parse().map_err(|_| DiceParsingError::InvalidAmount)?;

    if target <= 100 {
        Ok(DiceRoll {
            target: target,
            modifier: modifier,
        })
    } else {
        Err(DiceParsingError::InvalidAmount)
    }
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

    #[test]
    fn regular_roll_accepts_single_number() {
        let result = parse_regular_roll("60");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                target: 60,
                modifier: DiceRollModifier::Normal
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_two_bonus() {
        let result = parse_regular_roll("60bb");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                target: 60,
                modifier: DiceRollModifier::TwoBonus
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_one_bonus() {
        let result = parse_regular_roll("60b");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                target: 60,
                modifier: DiceRollModifier::OneBonus
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_two_penalty() {
        let result = parse_regular_roll("60pp");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                target: 60,
                modifier: DiceRollModifier::TwoPenalty
            },
            result.unwrap()
        );
    }

    #[test]
    fn regular_roll_accepts_one_penalty() {
        let result = parse_regular_roll("60p");
        assert!(result.is_ok());
        assert_eq!(
            DiceRoll {
                target: 60,
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

        assert!(parse_regular_roll("60bb     ").is_ok());
        assert!(parse_regular_roll("   60bb").is_ok());
        assert!(parse_regular_roll("   60bb    ").is_ok());

        assert!(parse_regular_roll("60b     ").is_ok());
        assert!(parse_regular_roll("   60b").is_ok());
        assert!(parse_regular_roll("   60b    ").is_ok());

        assert!(parse_regular_roll("60pp     ").is_ok());
        assert!(parse_regular_roll("   60pp").is_ok());
        assert!(parse_regular_roll("   60pp    ").is_ok());

        assert!(parse_regular_roll("60p     ").is_ok());
        assert!(parse_regular_roll("   60p").is_ok());
        assert!(parse_regular_roll("   60p    ").is_ok());
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
    fn regular_roll_rejects_big_numbers() {
        assert!(parse_regular_roll("3000").is_err());
    }

    #[test]
    fn advancement_roll_rejects_big_numbers() {
        assert!(parse_advancement_roll("3000").is_err());
    }

    #[test]
    fn regular_roll_rejects_invalid_input() {
        assert!(parse_regular_roll("abc").is_err());
    }

    #[test]
    fn advancement_roll_rejects_invalid_input() {
        assert!(parse_advancement_roll("abc").is_err());
    }
}
