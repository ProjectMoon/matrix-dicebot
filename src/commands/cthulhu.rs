use super::{Command, Execution};
use crate::context::Context;
use crate::cthulhu::dice::{regular_roll, AdvancementRoll, DiceRoll, DiceRollWithContext};
use async_trait::async_trait;

pub struct CthRoll(pub DiceRoll);

#[async_trait]
impl Command for CthRoll {
    fn name(&self) -> &'static str {
        "roll percentile pool"
    }

    async fn execute(&self, ctx: &Context<'_>) -> Execution {
        let roll_with_ctx = DiceRollWithContext(&self.0, ctx);
        let roll = regular_roll(&roll_with_ctx).await;

        let (plain, html) = match roll {
            Ok(executed_roll) => {
                let plain = format!("Roll: {}\nResult: {}", executed_roll, executed_roll.roll);
                let html = format!(
                    "<p><strong>Roll:</strong> {}</p><p><strong>Result</strong>: {}</p>",
                    executed_roll, executed_roll.roll
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

pub struct CthAdvanceRoll(pub AdvancementRoll);

#[async_trait]
impl Command for CthAdvanceRoll {
    fn name(&self) -> &'static str {
        "roll percentile pool"
    }

    async fn execute(&self, _ctx: &Context<'_>) -> Execution {
        //TODO this will be converted to a result when supporting variables.
        let roll = self.0.roll();
        let plain = format!("Roll: {}\nResult: {}", self.0, roll);
        let html = format!(
            "<p><strong>Roll:</strong> {}</p><p><strong>Result</strong>: {}</p>",
            self.0, roll
        );

        Execution { plain, html }
    }
}
