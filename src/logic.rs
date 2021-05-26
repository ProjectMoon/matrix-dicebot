use crate::error::{BotError, DiceRollingError};
use crate::parser::dice::{Amount, Element};
use crate::{context::Context, models::Account};
use crate::{
    db::{sqlite::Database, Users, Variables},
    models::TransientUser,
};
use argon2::{self, Config, Error as ArgonError};
use futures::stream::{self, StreamExt, TryStreamExt};
use rand::Rng;
use std::slice;

/// Calculate the amount of dice to roll by consulting the database
/// and replacing variables with corresponding the amount. Errors out
/// if it cannot find a variable defined, or if the database errors.
pub async fn calculate_single_die_amount(
    amount: &Amount,
    ctx: &Context<'_>,
) -> Result<i32, BotError> {
    calculate_dice_amount(slice::from_ref(amount), ctx).await
}

/// Calculate the amount of dice to roll by consulting the database
/// and replacing variables with corresponding amounts. Errors out if
/// it cannot find a variable defined, or if the database errors.
pub async fn calculate_dice_amount(amounts: &[Amount], ctx: &Context<'_>) -> Result<i32, BotError> {
    let stream = stream::iter(amounts);
    let variables = &ctx
        .db
        .get_user_variables(&ctx.username, ctx.room_id().as_str())
        .await?;

    use DiceRollingError::VariableNotFound;
    let dice_amount: i32 = stream
        .then(|amount| async move {
            match &amount.element {
                Element::Number(num_dice) => Ok(num_dice * amount.operator.mult()),
                Element::Variable(variable) => variables
                    .get(variable)
                    .ok_or_else(|| VariableNotFound(variable.clone()))
                    .map(|i| *i),
            }
        })
        .try_fold(0, |total, num_dice| async move { Ok(total + num_dice) })
        .await?;

    Ok(dice_amount)
}

/// Hash a password using the argon2 algorithm with a 16 byte salt.
pub(crate) fn hash_password(raw_password: &str) -> Result<String, ArgonError> {
    let salt = rand::thread_rng().gen::<[u8; 16]>();
    let config = Config::default();
    argon2::hash_encoded(raw_password.as_bytes(), &salt, &config)
}

pub(crate) async fn get_account(db: &Database, username: &str) -> Result<Account, BotError> {
    Ok(db
        .get_user(username)
        .await?
        .map(|user| Account::Registered(user))
        .unwrap_or_else(|| {
            Account::Transient(TransientUser {
                username: username.to_owned(),
            })
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Users;
    use crate::models::{AccountStatus, User};

    async fn create_db() -> Database {
        let db_path = tempfile::NamedTempFile::new_in(".").unwrap();
        crate::db::sqlite::migrator::migrate(db_path.path().to_str().unwrap())
            .await
            .unwrap();

        Database::new(db_path.path().to_str().unwrap())
            .await
            .unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn get_account_no_user_exists() {
        let db = create_db().await;

        let account = get_account(&db, "@test:example.com")
            .await
            .expect("Account retrieval didn't work");

        assert!(matches!(account, Account::Transient(_)));

        let user = account.transient_user().unwrap();
        assert_eq!(user.username, "@test:example.com");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn get_or_create_user_when_user_exists() {
        let db = create_db().await;

        let user = User {
            username: "myuser".to_string(),
            password: Some("abc".to_string()),
            account_status: AccountStatus::Registered,
            active_room: Some("myroom".to_string()),
        };

        let insert_result = db.upsert_user(&user).await;
        assert!(insert_result.is_ok());

        let account = get_account(&db, "myuser")
            .await
            .expect("Account retrieval did not work");

        assert!(matches!(account, Account::Registered(_)));

        let user_again = account.registered_user().unwrap();
        assert_eq!(user, user_again);
    }
}
