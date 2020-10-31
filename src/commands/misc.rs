use super::{Command, Execution};
use crate::context::Context;
use crate::help::HelpTopic;
use async_trait::async_trait;

pub struct HelpCommand(pub Option<HelpTopic>);

#[async_trait]
impl Command for HelpCommand {
    fn name(&self) -> &'static str {
        "help information"
    }

    async fn execute(&self, _ctx: &Context) -> Execution {
        let help = match &self.0 {
            Some(topic) => topic.message(),
            _ => "There is no help for this topic",
        };

        let plain = format!("Help: {}", help);
        let html = format!("<p><strong>Help:</strong> {}", help.replace("\n", "<br/>"));
        Execution { plain, html }
    }
}
