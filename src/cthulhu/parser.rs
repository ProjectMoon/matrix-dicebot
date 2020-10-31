use super::dice::{AdvancementRoll, DiceRoll};
use crate::error::BotError;
use combine::error::StringStreamError;
use combine::parser::char::{digit, letter, spaces, string};
use combine::{choice, count, many, many1, one_of, Parser};

pub fn parse_roll(input: &str) -> Result<DiceRoll, ParsingError> {
    Ok(DiceRoll {
        target: 50,
        modifier: DiceRollModifier::Normal,
    })
}

pub fn parse_advancement_roll(input: &str) -> Result<AdvancementRoll, ParsingError> {
    Ok(AdvancementRoll { existing_skill: 50 })
}
