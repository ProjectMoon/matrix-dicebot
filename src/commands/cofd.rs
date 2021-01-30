use super::{Command, CommandResult, Execution};
use crate::cofd::dice::{roll_pool, DicePool, DicePoolWithContext};
use crate::context::Context;
use async_trait::async_trait;

pub struct PoolRollCommand(pub DicePool);

#[async_trait]
impl Command for PoolRollCommand {
    fn name(&self) -> &'static str {
        "roll dice pool"
    }

    async fn execute(&self, ctx: &Context<'_>) -> CommandResult {
        let pool_with_ctx = DicePoolWithContext(&self.0, ctx);
        let rolled_pool = roll_pool(&pool_with_ctx).await?;

        let plain = format!("Pool: {}\nResult: {}", rolled_pool, rolled_pool.roll);
        let html = format!(
            "<p><strong>Pool:</strong> {}</p><p><strong>Result</strong>: {}</p>",
            rolled_pool, rolled_pool.roll
        );

        Execution::new(plain, html)
    }
}
