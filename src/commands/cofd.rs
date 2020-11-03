use super::{Command, Execution};
use crate::cofd::dice::{roll_pool, DicePool, DicePoolWithContext};
use crate::context::Context;
use async_trait::async_trait;

pub struct PoolRollCommand(pub DicePool);

#[async_trait]
impl Command for PoolRollCommand {
    fn name(&self) -> &'static str {
        "roll dice pool"
    }

    async fn execute(&self, ctx: &Context<'_>) -> Execution {
        let pool_with_ctx = DicePoolWithContext(&self.0, ctx);
        let roll_result = roll_pool(&pool_with_ctx).await;

        let (plain, html) = match roll_result {
            Ok(rolled_pool) => {
                let plain = format!("Pool: {}\nResult: {}", rolled_pool, rolled_pool.roll);
                let html = format!(
                    "<p><strong>Pool:</strong> {}</p><p><strong>Result</strong>: {}</p>",
                    rolled_pool, rolled_pool.roll
                );
                (plain, html)
            }
            Err(e) => {
                let plain = format!("Error: {}", e);
                let html = format!("<p><strong>Error:</strong> {}</p>", e);
                (plain, html)
            }
        };

        Execution { plain, html }
    }
}
