/**
 * In addition to the terms of the AGPL, this file is governed by the
 * terms of the MIT license, from the original axfive-matrix-dicebot
 * project.
 */
use crate::basic::dice;
use rand::prelude::*;
use std::fmt;
use std::ops::{Deref, DerefMut};

pub trait Roll {
    type Output;

    fn roll(&self) -> Self::Output;
}

pub trait Rolled {
    fn rolled_value(&self) -> i32;
}

#[derive(Debug, PartialEq, Eq, Clone)]
// array of rolls in order, how many dice to keep, and how many to drop
pub struct DiceRoll (pub Vec<u32>, usize, usize);

impl DiceRoll {
    pub fn rolls(&self) -> &[u32] {
        &self.0
    }

    pub fn keep(&self) -> usize {
        self.1
    }

    pub fn drop(&self) -> usize {
        self.2
    }

    // only count kept dice in total
    pub fn total(&self) -> u32 {
        self.0[self.2..self.1].iter().sum()
    }
}

impl Rolled for DiceRoll {
    fn rolled_value(&self) -> i32 {
        self.total() as i32
    }
}

impl fmt::Display for DiceRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rolled_value())?;
        let rolls = self.rolls();
        let keep  = self.keep();
        let drop  = self.drop();
        let mut iter = rolls.iter().enumerate();
        if let Some(first) = iter.next() {
            if drop != 0 {
                write!(f, " ([{}]", first.1)?;
            } else {
                write!(f, " ({}", first.1)?;
            }
            for roll in iter {
                if roll.0 >= keep || roll.0 < drop {
                    write!(f, " + [{}]", roll.1)?;
                } else {
                    write!(f, " + {}", roll.1)?;
                }
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl Roll for dice::Dice {
    type Output = DiceRoll;

    fn roll(&self) -> DiceRoll {
        let mut rng = rand::thread_rng();
        let mut rolls: Vec<_> = (0..self.count)
            .map(|_| rng.gen_range(1..=self.sides))
            .collect();
        // sort rolls in descending order
        rolls.sort_by(|a, b| b.cmp(a));

        DiceRoll(rolls,self.keep as usize, self.drop as usize)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ElementRoll {
    Dice(DiceRoll),
    Bonus(u32),
}

impl Rolled for ElementRoll {
    fn rolled_value(&self) -> i32 {
        match self {
            ElementRoll::Dice(d) => d.rolled_value(),
            ElementRoll::Bonus(b) => *b as i32,
        }
    }
}

impl Roll for dice::Element {
    type Output = ElementRoll;

    fn roll(&self) -> ElementRoll {
        match self {
            dice::Element::Dice(d) => ElementRoll::Dice(d.roll()),
            dice::Element::Bonus(b) => ElementRoll::Bonus(*b),
        }
    }
}

impl fmt::Display for ElementRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElementRoll::Dice(d) => write!(f, "{}", d),
            ElementRoll::Bonus(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SignedElementRoll {
    Positive(ElementRoll),
    Negative(ElementRoll),
}

impl Rolled for SignedElementRoll {
    fn rolled_value(&self) -> i32 {
        match self {
            SignedElementRoll::Positive(e) => e.rolled_value(),
            SignedElementRoll::Negative(e) => -e.rolled_value(),
        }
    }
}

impl Roll for dice::SignedElement {
    type Output = SignedElementRoll;

    fn roll(&self) -> SignedElementRoll {
        match self {
            dice::SignedElement::Positive(e) => SignedElementRoll::Positive(e.roll()),
            dice::SignedElement::Negative(e) => SignedElementRoll::Negative(e.roll()),
        }
    }
}

impl fmt::Display for SignedElementRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignedElementRoll::Positive(e) => write!(f, "{}", e),
            SignedElementRoll::Negative(e) => write!(f, "-{}", e),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ElementExpressionRoll(Vec<SignedElementRoll>);

impl Deref for ElementExpressionRoll {
    type Target = Vec<SignedElementRoll>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ElementExpressionRoll {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Rolled for ElementExpressionRoll {
    fn rolled_value(&self) -> i32 {
        self.iter().map(Rolled::rolled_value).sum()
    }
}

impl Roll for dice::ElementExpression {
    type Output = ElementExpressionRoll;

    fn roll(&self) -> ElementExpressionRoll {
        ElementExpressionRoll(self.iter().map(Roll::roll).collect())
    }
}

impl fmt::Display for ElementExpressionRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.len() > 1 {
            write!(f, "{}", self.rolled_value())?;
            let mut iter = self.0.iter();
            if let Some(first) = iter.next() {
                write!(f, " ({}", first)?;
                for roll in iter {
                    match roll {
                        SignedElementRoll::Positive(e) => write!(f, " + {}", e)?,
                        SignedElementRoll::Negative(e) => write!(f, " - {}", e)?,
                    }
                }
                write!(f, ")")?;
            }
            Ok(())
        } else if self.len() > 0 {
            // For a single item, just show the inner item to avoid redundancy
            let first = self.0.iter().next().unwrap();
            write!(f, "{}", first)
        } else {
            write!(f, "0")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dice_roll_display_test() {
        assert_eq!(DiceRoll(vec![1, 3, 4], 3, 0).to_string(), "8 (1 + 3 + 4)");
        assert_eq!(DiceRoll(vec![], 0, 0).to_string(), "0");
        assert_eq!(
            DiceRoll(vec![4, 7, 2, 10], 4, 0).to_string(),
            "23 (4 + 7 + 2 + 10)"
        );
        assert_eq!(
            DiceRoll(vec![20, 13, 11, 10], 3, 0).to_string(),
            "44 (20 + 13 + 11 + [10])"
        );
        assert_eq!(
            DiceRoll(vec![20, 13, 11, 10], 4, 1).to_string(),
            "34 ([20] + 13 + 11 + 10)"
        );
    }

    #[test]
    fn element_roll_display_test() {
        assert_eq!(
            ElementRoll::Dice(DiceRoll(vec![1, 3, 4], 3, 0)).to_string(),
            "8 (1 + 3 + 4)"
        );
        assert_eq!(ElementRoll::Bonus(7).to_string(), "7");
    }

    #[test]
    fn signed_element_roll_display_test() {
        assert_eq!(
            SignedElementRoll::Positive(ElementRoll::Dice(DiceRoll(vec![1, 3, 4], 3, 0))).to_string(),
            "8 (1 + 3 + 4)"
        );
        assert_eq!(
            SignedElementRoll::Negative(ElementRoll::Dice(DiceRoll(vec![1, 3, 4], 3, 0))).to_string(),
            "-8 (1 + 3 + 4)"
        );
        assert_eq!(
            SignedElementRoll::Positive(ElementRoll::Bonus(7)).to_string(),
            "7"
        );
        assert_eq!(
            SignedElementRoll::Negative(ElementRoll::Bonus(7)).to_string(),
            "-7"
        );
    }

    #[test]
    fn element_expression_roll_display_test() {
        assert_eq!(
            ElementExpressionRoll(vec![SignedElementRoll::Positive(ElementRoll::Dice(
                DiceRoll(vec![1, 3, 4], 3, 0)
            )),])
            .to_string(),
            "8 (1 + 3 + 4)"
        );
        assert_eq!(
            ElementExpressionRoll(vec![SignedElementRoll::Negative(ElementRoll::Dice(
                DiceRoll(vec![1, 3, 4], 3, 0)
            )),])
            .to_string(),
            "-8 (1 + 3 + 4)"
        );
        assert_eq!(
            ElementExpressionRoll(vec![SignedElementRoll::Positive(ElementRoll::Bonus(7)),])
                .to_string(),
            "7"
        );
        assert_eq!(
            ElementExpressionRoll(vec![SignedElementRoll::Negative(ElementRoll::Bonus(7)),])
                .to_string(),
            "-7"
        );
        assert_eq!(
            ElementExpressionRoll(vec![
                SignedElementRoll::Positive(ElementRoll::Dice(DiceRoll(vec![1, 3, 4], 3, 0))),
                SignedElementRoll::Negative(ElementRoll::Dice(DiceRoll(vec![1, 2], 2, 0))),
                SignedElementRoll::Positive(ElementRoll::Bonus(4)),
                SignedElementRoll::Negative(ElementRoll::Bonus(7)),
            ])
            .to_string(),
            "2 (8 (1 + 3 + 4) - 3 (1 + 2) + 4 - 7)"
        );
        assert_eq!(
            ElementExpressionRoll(vec![
                SignedElementRoll::Negative(ElementRoll::Dice(DiceRoll(vec![1, 3, 4], 3, 0))),
                SignedElementRoll::Positive(ElementRoll::Dice(DiceRoll(vec![1, 2], 2, 0))),
                SignedElementRoll::Negative(ElementRoll::Bonus(4)),
                SignedElementRoll::Positive(ElementRoll::Bonus(7)),
            ])
            .to_string(),
            "-2 (-8 (1 + 3 + 4) + 3 (1 + 2) - 4 + 7)"
        );
        assert_eq!(
            ElementExpressionRoll(vec![
                SignedElementRoll::Negative(ElementRoll::Dice(DiceRoll(vec![4, 3, 1], 3, 0))),
                SignedElementRoll::Positive(ElementRoll::Dice(DiceRoll(vec![12, 2], 1, 0))),
                SignedElementRoll::Negative(ElementRoll::Bonus(4)),
                SignedElementRoll::Positive(ElementRoll::Bonus(7)),
            ])
            .to_string(),
            "7 (-8 (4 + 3 + 1) + 12 (12 + [2]) - 4 + 7)"
        );
        assert_eq!(
            ElementExpressionRoll(vec![
                SignedElementRoll::Negative(ElementRoll::Dice(DiceRoll(vec![4, 3, 1], 3, 1))),
                SignedElementRoll::Positive(ElementRoll::Dice(DiceRoll(vec![12, 2], 2, 0))),
                SignedElementRoll::Negative(ElementRoll::Bonus(4)),
                SignedElementRoll::Positive(ElementRoll::Bonus(7)),
            ])
            .to_string(),
            "13 (-4 ([4] + 3 + 1) + 14 (12 + 2) - 4 + 7)"
        );
    }
}
