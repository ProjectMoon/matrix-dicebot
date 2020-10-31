use super::{Command, Execution};
use crate::context::Context;
use crate::cthulhu::dice::{AdvancementRoll, DiceRoll};
use async_trait::async_trait;

pub struct CthRoll(pub DiceRoll);

#[async_trait]
impl Command for CthRoll {
    fn name(&self) -> &'static str {
        "roll percentile pool"
    }

    async fn execute(&self, _ctx: &Context) -> Execution {
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

pub struct CthAdvanceRoll(pub AdvancementRoll);

#[async_trait]
impl Command for CthAdvanceRoll {
    fn name(&self) -> &'static str {
        "roll percentile pool"
    }

    async fn execute(&self, _ctx: &Context) -> Execution {
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
