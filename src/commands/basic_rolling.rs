use super::{Command, Execution};
use crate::context::Context;
use crate::dice::ElementExpression;
use crate::roll::Roll;
use async_trait::async_trait;

pub struct RollCommand(pub ElementExpression);

#[async_trait]
impl Command for RollCommand {
    fn name(&self) -> &'static str {
        "roll regular dice"
    }

    async fn execute(&self, _ctx: &Context<'_>) -> Execution {
        let roll = self.0.roll();
        let plain = format!("Dice: {}\nResult: {}", self.0, roll);
        let html = format!(
            "<p><strong>Dice:</strong> {}</p><p><strong>Result</strong>: {}</p>",
            self.0, roll
        );
        Execution { plain, html }
    }
}
