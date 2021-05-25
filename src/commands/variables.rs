use super::{Command, Execution, ExecutionResult};
use crate::context::Context;
use crate::db::errors::DataError;
use crate::db::Variables;
use crate::error::BotError;
use async_trait::async_trait;
use std::convert::TryFrom;

pub struct GetAllVariablesCommand;

impl From<GetAllVariablesCommand> for Box<dyn Command> {
    fn from(cmd: GetAllVariablesCommand) -> Self {
        Box::new(cmd)
    }
}

impl TryFrom<&str> for GetAllVariablesCommand {
    type Error = BotError;

    fn try_from(_: &str) -> Result<Self, Self::Error> {
        Ok(GetAllVariablesCommand)
    }
}

#[async_trait]
impl Command for GetAllVariablesCommand {
    fn name(&self) -> &'static str {
        "get all variables"
    }

    fn is_secure(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let variables = ctx
            .db
            .get_user_variables(&ctx.username, ctx.room_id().as_str())
            .await?;

        let mut variable_list: Vec<String> = variables
            .into_iter()
            .map(|(name, value)| format!(" - {} = {}", name, value))
            .collect();

        variable_list.sort();

        let value = variable_list.join("\n");
        let html = format!(
            "<strong>Variables:</strong><br/>{}",
            value.replace("\n", "<br/>")
        );

        Execution::success(html)
    }
}

pub struct GetVariableCommand(pub String);

impl From<GetVariableCommand> for Box<dyn Command> {
    fn from(cmd: GetVariableCommand) -> Self {
        Box::new(cmd)
    }
}

impl TryFrom<&str> for GetVariableCommand {
    type Error = BotError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Ok(GetVariableCommand(input.to_owned()))
    }
}

#[async_trait]
impl Command for GetVariableCommand {
    fn name(&self) -> &'static str {
        "retrieve variable value"
    }

    fn is_secure(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let name = &self.0;
        let result = ctx
            .db
            .get_user_variable(&ctx.username, ctx.room_id().as_str(), name)
            .await;

        let value = match result {
            Ok(num) => format!("{} = {}", name, num),
            Err(DataError::KeyDoesNotExist(_)) => format!("{} is not set", name),
            Err(e) => return Err(e.into()),
        };

        let html = format!("<strong>Variable:</strong> {}", value);
        Execution::success(html)
    }
}

pub struct SetVariableCommand(pub String, pub i32);

impl From<SetVariableCommand> for Box<dyn Command> {
    fn from(cmd: SetVariableCommand) -> Self {
        Box::new(cmd)
    }
}

impl TryFrom<&str> for SetVariableCommand {
    type Error = BotError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let (variable_name, value) = crate::parser::variables::parse_set_variable(input)?;
        Ok(SetVariableCommand(variable_name, value))
    }
}

#[async_trait]
impl Command for SetVariableCommand {
    fn name(&self) -> &'static str {
        "set variable value"
    }

    fn is_secure(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let name = &self.0;
        let value = self.1;

        ctx.db
            .set_user_variable(&ctx.username, ctx.room_id().as_str(), name, value)
            .await?;

        let content = format!("{} = {}", name, value);
        let html = format!("<strong>Set Variable:</strong> {}", content);
        Execution::success(html)
    }
}

pub struct DeleteVariableCommand(pub String);

impl From<DeleteVariableCommand> for Box<dyn Command> {
    fn from(cmd: DeleteVariableCommand) -> Self {
        Box::new(cmd)
    }
}

impl TryFrom<&str> for DeleteVariableCommand {
    type Error = BotError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Ok(DeleteVariableCommand(input.to_owned()))
    }
}

#[async_trait]
impl Command for DeleteVariableCommand {
    fn name(&self) -> &'static str {
        "delete variable"
    }

    fn is_secure(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let name = &self.0;
        let result = ctx
            .db
            .delete_user_variable(&ctx.username, ctx.room_id().as_str(), name)
            .await;

        let value = match result {
            Ok(()) => format!("{} now unset", name),
            Err(DataError::KeyDoesNotExist(_)) => format!("{} is not currently set", name),
            Err(e) => return Err(e.into()),
        };

        let html = format!("<strong>Remove Variable:</strong> {}", value);
        Execution::success(html)
    }
}
