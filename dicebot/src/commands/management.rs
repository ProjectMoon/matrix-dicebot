use super::{Command, Execution, ExecutionResult};
use crate::db::Users;
use crate::error::BotError::{AccountDoesNotExist, PasswordCreationError};
use crate::logic::hash_password;
use crate::models::{AccountStatus, User};
use crate::{context::Context, error::BotError};
use async_trait::async_trait;
use std::convert::{Into, TryFrom};

pub struct RegisterCommand;

impl TryFrom<String> for RegisterCommand {
    type Error = BotError;

    fn try_from(_: String) -> Result<Self, Self::Error> {
        Ok(RegisterCommand)
    }
}

#[async_trait]
impl Command for RegisterCommand {
    fn name(&self) -> &'static str {
        "register user account"
    }

    fn is_secure(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        if ctx.account.is_registered() {
            return Err(BotError::AccountAlreadyExists);
        }

        let user = User {
            username: ctx.username.to_owned(),
            password: None,
            account_status: AccountStatus::Registered,
            ..Default::default()
        };

        ctx.db.upsert_user(&user).await?;

        Execution::success(format!(
            "User account {} registered for bot commands.",
            ctx.username
        ))
    }
}

pub struct UnlinkCommand(pub String);

impl TryFrom<String> for UnlinkCommand {
    type Error = BotError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(UnlinkCommand(value))
    }
}

#[async_trait]
impl Command for UnlinkCommand {
    fn name(&self) -> &'static str {
        "unlink user accountx from external applications"
    }

    fn is_secure(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let mut user = ctx
            .db
            .get_user(&ctx.username)
            .await?
            .ok_or(BotError::AccountDoesNotExist)?;

        user.password = None;
        ctx.db.upsert_user(&user).await?;

        Execution::success(format!(
            "Accounted {} is now inaccessible to external applications.",
            ctx.username
        ))
    }
}

pub struct LinkCommand(pub String);

impl TryFrom<String> for LinkCommand {
    type Error = BotError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(LinkCommand(value))
    }
}

#[async_trait]
impl Command for LinkCommand {
    fn name(&self) -> &'static str {
        "link user account to external applications"
    }

    fn is_secure(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let mut user = ctx
            .db
            .get_user(&ctx.username)
            .await?
            .ok_or(BotError::AccountDoesNotExist)?;

        let pw_hash = hash_password(&self.0).map_err(|e| PasswordCreationError(e))?;
        user.password = Some(pw_hash);
        ctx.db.upsert_user(&user).await?;

        Execution::success(format!(
            "Accounted now available for external use. Please log in to \
             external applications with username {} and the password you set.",
            ctx.username
        ))
    }
}

pub struct CheckCommand;

impl TryFrom<String> for CheckCommand {
    type Error = BotError;

    fn try_from(_: String) -> Result<Self, Self::Error> {
        Ok(CheckCommand)
    }
}

#[async_trait]
impl Command for CheckCommand {
    fn name(&self) -> &'static str {
        "check user account status"
    }

    fn is_secure(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &Context<'_>) -> ExecutionResult {
        let user = ctx.db.get_user(&ctx.username).await?;

        match user {
            Some(user) => match user.password {
                Some(_) => Execution::success(
                    "Account exists, and is available to external applications with a password. \
                     If you forgot your password, change it with !link."
                        .to_string(),
                ),
                None => Execution::success(
                    "Account exists, but is not available to external applications.".to_string(),
                ),
            },
            None => Execution::success(
                "No account registered. Only simple commands in public rooms are available."
                    .to_string(),
            ),
        }
    }
}

pub struct UnregisterCommand;

impl TryFrom<String> for UnregisterCommand {
    type Error = BotError;

    fn try_from(_: String) -> Result<Self, Self::Error> {
        Ok(UnregisterCommand)
    }
}

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
