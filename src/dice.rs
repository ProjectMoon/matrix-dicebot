pub mod parser;

use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Dice {
    pub(crate) count: u32,
    pub(crate) sides: u32,
}

impl Dice {
    fn new(count: u32, sides: u32) -> Dice {
        Dice { count, sides }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Element {
    Dice(Dice),
    Bonus(u32),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SignedElement {
    Positive(Element),
    Negative(Element),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ElementExpression(Vec<SignedElement>);

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
