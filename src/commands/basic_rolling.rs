use super::{Command, Execution, ExecutionResult};
use crate::basic::dice::ElementExpression;
use crate::basic::parser::parse_element_expression;
use crate::basic::roll::Roll;
use crate::context::Context;
use crate::error::BotError;
use async_trait::async_trait;
use nom::Err as NomErr;
use std::convert::TryFrom;

pub struct RollCommand(pub ElementExpression);

impl From<RollCommand> for Box<dyn Command> {
    fn from(cmd: RollCommand) -> Self {
        Box::new(cmd)
    }
}

impl TryFrom<String> for RollCommand {
    type Error = BotError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        let result = parse_element_expression(&input);
        match result {
            Ok((rest, expression)) if rest.len() == 0 => Ok(RollCommand(expression)),
            //"Legacy code boundary": translates Nom errors into BotErrors.
            Ok(_) => Err(BotError::NomParserIncomplete),
            Err(NomErr::Error(e)) => Err(BotError::NomParserError(e.1)),
            Err(NomErr::Failure(e)) => Err(BotError::NomParserError(e.1)),
            Err(NomErr::Incomplete(_)) => Err(BotError::NomParserIncomplete),
        }
    }
}

#[async_trait]
impl Command for RollCommand {
    fn name(&self) -> &'static str {
        "roll regular dice"
    }

    fn is_secure(&self) -> bool {
        false
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
