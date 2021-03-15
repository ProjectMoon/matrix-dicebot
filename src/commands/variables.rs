use super::{Command, Execution, ExecutionResult};
use crate::context::Context;
use crate::db::errors::DataError;
use crate::db::variables::UserAndRoom;
use async_trait::async_trait;

pub struct GetAllVariablesCommand;

#[async_trait]
impl Command for GetAllVariablesCommand {
    fn name(&self) -> &'static str {
        "get all variables"
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let key = UserAndRoom(&ctx.username, &ctx.room_id().as_str());
        let variables = ctx.db.variables.get_user_variables(&key)?;

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

#[async_trait]
impl Command for GetVariableCommand {
    fn name(&self) -> &'static str {
        "retrieve variable value"
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let name = &self.0;
        let key = UserAndRoom(&ctx.username, &ctx.room_id().as_str());
        let result = ctx.db.variables.get_user_variable(&key, name);

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

#[async_trait]
impl Command for SetVariableCommand {
    fn name(&self) -> &'static str {
        "set variable value"
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let name = &self.0;
        let value = self.1;
        let key = UserAndRoom(&ctx.username, ctx.room_id().as_str());

        ctx.db.variables.set_user_variable(&key, name, value)?;

        let content = format!("{} = {}", name, value);
        let html = format!("<strong>Set Variable:</strong> {}", content);
        Execution::success(html)
    }
}

pub struct DeleteVariableCommand(pub String);

#[async_trait]
impl Command for DeleteVariableCommand {
    fn name(&self) -> &'static str {
        "delete variable"
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let name = &self.0;
        let key = UserAndRoom(&ctx.username, ctx.room_id().as_str());
        let result = ctx.db.variables.delete_user_variable(&key, name);

        let value = match result {
            Ok(()) => format!("{} now unset", name),
            Err(DataError::KeyDoesNotExist(_)) => format!("{} is not currently set", name),
            Err(e) => return Err(e.into()),
        };

        let html = format!("<strong>Remove Variable:</strong> {}", value);
        Execution::success(html)
    }
}
