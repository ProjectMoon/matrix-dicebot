use super::{Command, Execution, ExecutionResult};
use crate::cofd::dice::{roll_pool, DicePool, DicePoolWithContext};
use crate::context::Context;
use async_trait::async_trait;

pub struct PoolRollCommand(pub DicePool);

#[async_trait]
impl Command for PoolRollCommand {
    fn name(&self) -> &'static str {
        "roll dice pool"
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let pool_with_ctx = DicePoolWithContext(&self.0, ctx);
        let rolled_pool = roll_pool(&pool_with_ctx).await?;

        let html = format!(
            "<p><strong>Pool:</strong> {}</p><p><strong>Result</strong>: {}</p>",
            rolled_pool, rolled_pool.roll
        );

        Execution::success(html)
    }
}
