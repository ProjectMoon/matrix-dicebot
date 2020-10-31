use super::dice::{AdvancementRoll, DiceRoll, DiceRollModifier};
use crate::parser::DiceParsingError;

//TOOD convert these to use parse_amounts from the common dice code.

pub fn parse_regular_roll(input: &str) -> Result<DiceRoll, DiceParsingError> {
    let input = input.trim();
    let target: u32 = input.parse().map_err(|_| DiceParsingError::InvalidAmount)?;

    if target <= 100 {
        Ok(DiceRoll {
            target: target,
            modifier: DiceRollModifier::Normal,
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
    fn regular_roll_accepts_whitespacen() {
        assert!(parse_regular_roll("60     ").is_ok());
        assert!(parse_regular_roll("   60").is_ok());
        assert!(parse_regular_roll("   60    ").is_ok());
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
