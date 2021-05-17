use crate::db::sqlite::errors::DataError;
use crate::db::sqlite::Rooms;
use crate::error::BotError;
use crate::matrix;
use crate::models::RoomInfo;
use futures::stream::{self, StreamExt, TryStreamExt};
use matrix_sdk::{self, identifiers::RoomId, Client};

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
