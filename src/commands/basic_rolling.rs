use super::{Command, Execution, ExecutionResult};
use crate::basic::dice::ElementExpression;
use crate::basic::roll::Roll;
use crate::context::Context;
use async_trait::async_trait;

pub struct RollCommand(pub ElementExpression);

#[async_trait]
impl Command for RollCommand {
    fn name(&self) -> &'static str {
        "roll regular dice"
    }

    async fn execute(&self, _ctx: &Context<'_>) -> ExecutionResult {
        let roll = self.0.roll();
        let html = format!(
            "<strong>Dice:</strong> {}</p><p><strong>Result</strong>: {}",
            self.0, roll
        );

        Execution::success(html)
    }
}
