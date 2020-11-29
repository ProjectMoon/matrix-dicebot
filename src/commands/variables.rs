use super::{Command, Execution};
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

    async fn execute(&self, ctx: &Context<'_>) -> Execution {
        let key = UserAndRoom(&ctx.username, &ctx.room.room_id.as_str());
        let result = ctx.db.variables.get_user_variables(&key);

        let value = match result {
            Ok(variables) => {
                let mut variable_list = variables
                    .into_iter()
                    .map(|(name, value)| format!(" - {} = {}", name, value))
                    .collect::<Vec<_>>();

                variable_list.sort();
                variable_list.join("\n")
            }
            Err(e) => format!("error getting variables: {}", e),
        };

        let plain = format!("Variables:\n{}", value);
        let html = format!(
            "<p><strong>Variables:</strong><br/>{}",
            value.replace("\n", "<br/>")
        );
        Execution { plain, html }
    }
}

pub struct GetVariableCommand(pub String);

#[async_trait]
impl Command for GetVariableCommand {
    fn name(&self) -> &'static str {
        "retrieve variable value"
    }

    async fn execute(&self, ctx: &Context<'_>) -> Execution {
        let name = &self.0;
        let key = UserAndRoom(&ctx.username, &ctx.room.room_id.as_str());
        let result = ctx.db.variables.get_user_variable(&key, name);

        let value = match result {
            Ok(num) => format!("{} = {}", name, num),
            Err(DataError::KeyDoesNotExist(_)) => format!("{} is not set", name),
            Err(e) => format!("error getting {}: {}", name, e),
        };

        let plain = format!("Variable: {}", value);
        let html = format!("<p><strong>Variable:</strong> {}", value);
        Execution { plain, html }
    }
}

pub struct SetVariableCommand(pub String, pub i32);

#[async_trait]
impl Command for SetVariableCommand {
    fn name(&self) -> &'static str {
        "set variable value"
    }

    async fn execute(&self, ctx: &Context<'_>) -> Execution {
        let name = &self.0;
        let value = self.1;
        let key = UserAndRoom(&ctx.username, ctx.room.room_id.as_str());
        let result = ctx.db.variables.set_user_variable(&key, name, value);

        let content = match result {
            Ok(_) => format!("{} = {}", name, value),
            Err(e) => format!("error setting {}: {}", name, e),
        };

        let plain = format!("Set Variable: {}", content);
        let html = format!("<p><strong>Set Variable:</strong> {}", content);
        Execution { plain, html }
    }
}

pub struct DeleteVariableCommand(pub String);

#[async_trait]
impl Command for DeleteVariableCommand {
    fn name(&self) -> &'static str {
        "delete variable"
    }

    async fn execute(&self, ctx: &Context<'_>) -> Execution {
        let name = &self.0;
        let key = UserAndRoom(&ctx.username, ctx.room.room_id.as_str());
        let result = ctx.db.variables.delete_user_variable(&key, name);

        let value = match result {
            Ok(()) => format!("{} now unset", name),
            Err(DataError::KeyDoesNotExist(_)) => format!("{} is not currently set", name),
            Err(e) => format!("error deleting {}: {}", name, e),
        };

        let plain = format!("Remove Variable: {}", value);
        let html = format!("<p><strong>Remove Variable:</strong> {}", value);
        Execution { plain, html }
    }
}
