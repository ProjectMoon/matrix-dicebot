/**
 * In addition to the terms of the AGPL, this file is governed by the
 * terms of the MIT license, from the original axfive-matrix-dicebot
 * project.
 */
use std::fmt;
use std::ops::{Deref, DerefMut};

//Old stuff, for regular dice rolling. To be moved elsewhere.

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Dice {
    pub(crate) count: u32,
    pub(crate) sides: u32,
    pub(crate) keep: u32,
    pub(crate) drop: u32,
}

impl fmt::Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.keep != self.count {
            write!(f, "{}d{}k{}", self.count, self.sides, self.keep)
        } else if self.drop != 0 {
            write!(f, "{}d{}d{}", self.count, self.sides, self.drop)
        } else {
            write!(f, "{}d{}", self.count, self.sides)
        }
    }
}

impl Dice {
    pub fn new(count: u32, sides: u32, keep: u32, drop: u32) -> Dice {
        Dice { count, sides, keep, drop }
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
