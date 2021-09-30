/**
 * In addition to the terms of the AGPL, this file is governed by the
 * terms of the MIT license, from the original axfive-matrix-dicebot
 * project.
 */
use std::fmt;
use std::ops::{Deref, DerefMut};

/// A basic dice roll, in XdY notation, like "1d4" or "3d6".
/// Optionally supports D&D advantage/disadvantge keep-or-drop
/// functionality.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Dice {
    pub(crate) count: u32,
    pub(crate) sides: u32,
    pub(crate) keep_drop: KeepOrDrop,
}

/// Enum indicating how to handle bonuses or penalties using extra
/// dice. If set to Keep, the roll will keep the highest X number of
/// dice in the roll, and add those together. If set to Drop, the
/// opposite is performed, and the lowest X number of dice are added
/// instead. If set to None, then all dice in the roll are added up as
/// normal.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeepOrDrop {
    /// Keep only the X highest dice for adding up to the total.
    Keep(u32),

    /// Keep only the X lowest dice (i.e. drop the highest) for adding
    /// up to the total.
    Drop(u32),

    /// Add up all dice in the roll for the total.
    None,
}

impl fmt::Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.keep_drop {
            KeepOrDrop::Keep(keep) => write!(f, "{}d{}k{}", self.count, self.sides, keep),
            KeepOrDrop::Drop(drop) => write!(f, "{}d{}dh{}", self.count, self.sides, drop),
            KeepOrDrop::None => write!(f, "{}d{}", self.count, self.sides),
        }
    }
}

impl Dice {
    pub fn new(count: u32, sides: u32, keep_drop: KeepOrDrop) -> Dice {
        Dice {
            count,
            sides,
            keep_drop,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Element {
    Dice(Dice),
    Bonus(u32),
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Element::Dice(d) => write!(f, "{}", d),
            Element::Bonus(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SignedElement {
    Positive(Element),
    Negative(Element),
}

impl fmt::Display for SignedElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignedElement::Positive(e) => write!(f, "{}", e),
            SignedElement::Negative(e) => write!(f, "-{}", e),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ElementExpression(pub Vec<SignedElement>);

impl Deref for ElementExpression {
    type Target = Vec<SignedElement>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ElementExpression {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for ElementExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter();
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
            for roll in iter {
                match roll {
                    SignedElement::Positive(e) => write!(f, " + {}", e)?,
                    SignedElement::Negative(e) => write!(f, " - {}", e)?,
                }
            }
        }
        Ok(())
    }
}
