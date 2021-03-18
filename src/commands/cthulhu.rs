use super::{Command, Execution, ExecutionResult};
use crate::context::Context;
use crate::cthulhu::dice::{
    advancement_roll, regular_roll, AdvancementRoll, AdvancementRollWithContext, DiceRoll,
    DiceRollWithContext,
};
use async_trait::async_trait;

pub struct CthRoll(pub DiceRoll);

#[async_trait]
impl Command for CthRoll {
    fn name(&self) -> &'static str {
        "roll percentile dice"
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let roll_with_ctx = DiceRollWithContext(&self.0, ctx);
        let executed_roll = regular_roll(&roll_with_ctx).await?;

        let html = format!(
            "<p><strong>Roll:</strong> {}</p><p><strong>Result</strong>: {}</p>",
            executed_roll, executed_roll.roll
        );

        Execution::success(html)
    }
}

pub struct CthAdvanceRoll(pub AdvancementRoll);

#[async_trait]
impl Command for CthAdvanceRoll {
    fn name(&self) -> &'static str {
        "roll skill advancement dice"
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let roll_with_ctx = AdvancementRollWithContext(&self.0, ctx);
        let executed_roll = advancement_roll(&roll_with_ctx).await?;
        let html = format!(
            "<p><strong>Roll:</strong> {}</p><p><strong>Result</strong>: {}</p>",
            executed_roll, executed_roll.roll
        );

        Execution::success(html)
    }
}
