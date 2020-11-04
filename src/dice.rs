pub mod parser;

use crate::context::Context;
use crate::db::variables::UserAndRoom;
use crate::error::BotError;
use crate::error::DiceRollingError;
use crate::parser::Amount;
use crate::parser::Element as NewElement;
use futures::stream::{self, StreamExt, TryStreamExt};
use std::fmt;
use std::ops::{Deref, DerefMut};

//New hotness
pub async fn calculate_dice_amount(amounts: &[Amount], ctx: &Context<'_>) -> Result<i32, BotError> {
    let stream = stream::iter(amounts);
    let key = UserAndRoom(&ctx.username, &ctx.room_id);
    let variables = &ctx.db.variables.get_user_variables(&key)?;

    use DiceRollingError::VariableNotFound;
    let dice_amount: Result<i32, BotError> = stream
        .then(|amount| async move {
            match &amount.element {
                NewElement::Number(num_dice) => Ok(*num_dice * amount.operator.mult()),
                NewElement::Variable(variable) => variables
                    .get(variable)
                    .ok_or(VariableNotFound(variable.clone().to_string()))
                    .map(|i| *i)
                    .map_err(|e| e.into()),
            }
        })
        .try_fold(0, |total, num_dice| async move { Ok(total + num_dice) })
        .await;

    dice_amount
}

//Old stuff, for regular dice rolling. To be moved elsewhere.

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Dice {
    pub(crate) count: u32,
    pub(crate) sides: u32,
}

impl fmt::Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}d{}", self.count, self.sides)
    }
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
