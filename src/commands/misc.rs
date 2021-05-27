use super::{Command, Execution, ExecutionResult};
use crate::context::Context;
use crate::error::BotError;
use crate::help::{parse_help_topic, HelpTopic};
use async_trait::async_trait;
use std::convert::TryFrom;

pub struct HelpCommand(pub Option<HelpTopic>);

impl From<HelpCommand> for Box<dyn Command> {
    fn from(cmd: HelpCommand) -> Self {
        Box::new(cmd)
    }
}

impl TryFrom<String> for HelpCommand {
    type Error = BotError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        let topic = parse_help_topic(&input);
        Ok(HelpCommand(topic))
    }
}

#[async_trait]
impl Command for HelpCommand {
    fn name(&self) -> &'static str {
        "help information"
    }

    fn is_secure(&self) -> bool {
        false
    }

    async fn execute(&self, _ctx: &Context<'_>) -> ExecutionResult {
        let help = match &self.0 {
            Some(topic) => topic.message(),
            _ => "There is no help for this topic",
        };

        let html = format!("<strong>Help:</strong> {}", help.replace("\n", "<br/>"));
        Execution::success(html)
    }
}
