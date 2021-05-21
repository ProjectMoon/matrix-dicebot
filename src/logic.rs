use crate::context::Context;
use crate::db::{Rooms, Variables};
use crate::error::{BotError, DiceRollingError};
use crate::matrix;
use crate::models::RoomInfo;
use crate::parser::dice::{Amount, Element};
use futures::stream::{self, StreamExt, TryStreamExt};
use matrix_sdk::{self, identifiers::RoomId, Client};
use std::slice;

/// Record the information about a room, including users in it.
pub async fn record_room_information(
    client: &Client,
    db: &crate::db::sqlite::Database,
    room_id: &RoomId,
    room_display_name: &str,
    our_username: &str,
) -> Result<(), BotError> {
    //Clear out any old room info first.
    db.clear_info(room_id.as_str()).await?;

    let room_id_str = room_id.as_str();
    let usernames = matrix::get_users_in_room(&client, &room_id).await?;

    let info = RoomInfo {
        room_id: room_id_str.to_owned(),
        room_name: room_display_name.to_owned(),
    };

    // TODO this and the username adding should be one whole
    // transaction in the db.
    db.insert_room_info(&info).await?;

    let filtered_usernames = usernames
        .into_iter()
        .filter(|username| username != our_username);

    // Async collect into vec of results, then use into_iter of result
    // to go to from Result<Vec<()>> to just Result<()>. Easier than
    // attempting to async-collect our way to a single Result<()>.
    stream::iter(filtered_usernames)
        .then(|username| async move {
            db.add_user_to_room(&username, &room_id_str)
                .await
                .map_err(|e| e.into())
        })
        .collect::<Vec<Result<(), BotError>>>()
        .await
        .into_iter()
        .collect()
}

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
