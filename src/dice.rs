use crate::context::Context;
use crate::db::sqlite::Variables;
use crate::db::variables::UserAndRoom;
use crate::error::BotError;
use crate::error::DiceRollingError;
use crate::parser::Amount;
use crate::parser::Element as NewElement;
use futures::stream::{self, StreamExt, TryStreamExt};
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
                NewElement::Number(num_dice) => Ok(num_dice * amount.operator.mult()),
                NewElement::Variable(variable) => variables
                    .get(variable)
                    .ok_or_else(|| VariableNotFound(variable.clone()))
                    .map(|i| *i),
            }
        })
        .try_fold(0, |total, num_dice| async move { Ok(total + num_dice) })
        .await?;

    Ok(dice_amount)
}
