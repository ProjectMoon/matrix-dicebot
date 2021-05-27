use super::{Command, Execution, ExecutionResult};
use crate::cofd::dice::{roll_pool, DicePool, DicePoolWithContext};
use crate::cofd::parser::{create_chance_die, parse_dice_pool};
use crate::context::Context;
use crate::error::BotError;
use async_trait::async_trait;
use std::convert::TryFrom;

pub struct PoolRollCommand(pub DicePool);

impl PoolRollCommand {
    pub fn chance_die() -> Result<PoolRollCommand, BotError> {
        let pool = create_chance_die()?;
        Ok(PoolRollCommand(pool))
    }
}

impl From<PoolRollCommand> for Box<dyn Command> {
    fn from(cmd: PoolRollCommand) -> Self {
        Box::new(cmd)
    }
}

impl TryFrom<String> for PoolRollCommand {
    type Error = BotError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        let pool = parse_dice_pool(&input)?;
        Ok(PoolRollCommand(pool))
    }
}

#[async_trait]
impl Command for PoolRollCommand {
    fn name(&self) -> &'static str {
        "roll dice pool"
    }

    fn is_secure(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let pool_with_ctx = DicePoolWithContext(&self.0, ctx);
        let rolled_pool = roll_pool(&pool_with_ctx).await?;

        let html = format!(
            "<strong>Pool:</strong> {}</p><p><strong>Result</strong>: {}",
            rolled_pool, rolled_pool.roll
        );

        Execution::success(html)
    }
}
