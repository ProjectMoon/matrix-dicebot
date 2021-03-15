use crate::context::Context;
use crate::db::variables::UserAndRoom;
use crate::error::BotError;
use crate::error::DiceRollingError;
use crate::parser::Amount;
use crate::parser::Element as NewElement;
use futures::stream::{self, StreamExt, TryStreamExt};

/// Calculate the amount of dice to roll by consulting the database
/// and replacing variables with corresponding amounts. Errors out if
/// it cannot find a variable defined, or if the database errors.
pub async fn calculate_dice_amount(amounts: &[Amount], ctx: &Context<'_>) -> Result<i32, BotError> {
    let stream = stream::iter(amounts);
    let key = UserAndRoom(&ctx.username, ctx.room_id().as_str());
    let variables = &ctx.db.variables.get_user_variables(&key)?;

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
