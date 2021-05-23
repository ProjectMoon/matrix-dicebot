use super::{Command, Execution, ExecutionResult};
use crate::context::Context;
use crate::db::Users;
use crate::error::BotError::{AccountDoesNotExist, AuthenticationError, PasswordCreationError};
use crate::logic::{hash_password, record_room_information};
use crate::models::User;
use async_trait::async_trait;
use matrix_sdk::identifiers::UserId;

pub struct ResyncCommand;

#[async_trait]
impl Command for ResyncCommand {
    fn name(&self) -> &'static str {
        "resync room information"
    }

    fn is_secure(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let our_username: Option<UserId> = ctx.matrix_client.user_id().await;
        let our_username: &str = our_username.as_ref().map_or("", UserId::as_str);

        record_room_information(
            ctx.matrix_client,
            &ctx.db,
            ctx.room_id(),
            &ctx.room.display_name,
            our_username,
        )
        .await?;

        let message = "Room information resynced.".to_string();
        Execution::success(message)
    }
}

pub struct RegisterCommand(pub String);

#[async_trait]
impl Command for RegisterCommand {
    fn name(&self) -> &'static str {
        "register user account"
    }

    fn is_secure(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let pw_hash = hash_password(&self.0).map_err(|e| PasswordCreationError(e))?;
        let user = User {
            username: ctx.username.to_owned(),
            password: pw_hash,
        };

        ctx.db.upsert_user(&user).await?;
        Execution::success(format!(
            "User account registered/updated. Please log in to external applications \
             with username {} and the password you set.",
            ctx.username
        ))
    }
}

pub struct CheckCommand(pub String);

#[async_trait]
impl Command for CheckCommand {
    fn name(&self) -> &'static str {
        "check user password"
    }

    fn is_secure(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let user = ctx.db.authenticate_user(&ctx.username, &self.0).await?;

        match user {
            Some(_) => Execution::success("Password is correct!".to_string()),
            None => Err(AuthenticationError.into()),
        }
    }
}

pub struct UnregisterCommand;

#[async_trait]
impl Command for UnregisterCommand {
    fn name(&self) -> &'static str {
        "unregister user account"
    }

    fn is_secure(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let user = ctx.db.get_user(&ctx.username).await?;

        match user {
            Some(_) => {
                ctx.db.delete_user(&ctx.username).await?;
                Execution::success("Your user account has been removed.".to_string())
            }
            None => Err(AccountDoesNotExist.into()),
        }
    }
}
