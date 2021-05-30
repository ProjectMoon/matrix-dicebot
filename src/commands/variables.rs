use super::{Command, Execution, ExecutionResult};
use crate::context::Context;
use crate::db::errors::DataError;
use crate::db::Variables;
use crate::error::BotError;
use async_trait::async_trait;
use std::convert::TryFrom;

pub struct GetAllVariablesCommand;

impl TryFrom<String> for GetAllVariablesCommand {
    type Error = BotError;

    fn try_from(_: String) -> Result<Self, Self::Error> {
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
            .get_user_variables(&ctx.username, ctx.active_room_id().as_str())
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

impl TryFrom<String> for GetVariableCommand {
    type Error = BotError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        Ok(GetVariableCommand(input))
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
            .get_user_variable(&ctx.username, ctx.active_room_id().as_str(), name)
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

impl TryFrom<String> for SetVariableCommand {
    type Error = BotError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        let (variable_name, value) = crate::parser::variables::parse_set_variable(&input)?;
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
            .set_user_variable(&ctx.username, ctx.active_room_id().as_str(), name, value)
            .await?;

        let content = format!("{} = {}", name, value);
        let html = format!("<strong>Set Variable:</strong> {}", content);
        Execution::success(html)
    }
}

pub struct DeleteVariableCommand(pub String);

impl TryFrom<String> for DeleteVariableCommand {
    type Error = BotError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        Ok(DeleteVariableCommand(input))
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
            .delete_user_variable(&ctx.username, ctx.active_room_id().as_str(), name)
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
