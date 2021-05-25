use super::{Command, Execution, ExecutionResult};
use crate::context::Context;
use crate::cthulhu::dice::{
    advancement_roll, regular_roll, AdvancementRoll, AdvancementRollWithContext, DiceRoll,
    DiceRollWithContext,
};
use crate::cthulhu::parser::{parse_advancement_roll, parse_regular_roll};
use crate::error::BotError;
use async_trait::async_trait;
use std::convert::TryFrom;

pub struct CthRoll(pub DiceRoll);

impl From<CthRoll> for Box<dyn Command> {
    fn from(cmd: CthRoll) -> Self {
        Box::new(cmd)
    }
}

impl TryFrom<&str> for CthRoll {
    type Error = BotError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let roll = parse_regular_roll(input)?;
        Ok(CthRoll(roll))
    }
}

#[async_trait]
impl Command for CthRoll {
    fn name(&self) -> &'static str {
        "roll percentile dice"
    }

    fn is_secure(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let roll_with_ctx = DiceRollWithContext(&self.0, ctx);
        let executed_roll = regular_roll(&roll_with_ctx).await?;

        let html = format!(
            "<strong>Roll:</strong> {}</p><p><strong>Result</strong>: {}",
            executed_roll, executed_roll.roll
        );

        Execution::success(html)
    }
}

pub struct CthAdvanceRoll(pub AdvancementRoll);

impl From<CthAdvanceRoll> for Box<dyn Command> {
    fn from(cmd: CthAdvanceRoll) -> Self {
        Box::new(cmd)
    }
}

impl TryFrom<&str> for CthAdvanceRoll {
    type Error = BotError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let roll = parse_advancement_roll(input)?;
        Ok(CthAdvanceRoll(roll))
    }
}

#[async_trait]
impl Command for CthAdvanceRoll {
    fn name(&self) -> &'static str {
        "roll skill advancement dice"
    }

    fn is_secure(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let roll_with_ctx = AdvancementRollWithContext(&self.0, ctx);
        let executed_roll = advancement_roll(&roll_with_ctx).await?;
        let html = format!(
            "<strong>Roll:</strong> {}</p><p><strong>Result</strong>: {}",
            executed_roll, executed_roll.roll
        );

        Execution::success(html)
    }
}
