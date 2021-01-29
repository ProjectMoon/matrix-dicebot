use crate::db::errors::DataError;
use crate::matrix;
use crate::models::RoomInfo;
use matrix_sdk::{self, identifiers::RoomId, Client};

/// Record the information about a room, including users in it.
pub async fn record_room_information(
    client: &Client,
    db: &crate::db::Database,
    room_id: &RoomId,
    room_display_name: &str,
    our_username: &str,
) -> Result<(), DataError> {
    let room_id_str = room_id.as_str();
    let usernames = matrix::get_users_in_room(&client, &room_id).await;

    let info = RoomInfo {
        room_id: room_id_str.to_owned(),
        room_name: room_display_name.to_owned(),
    };

    // TODO this and the username adding should be one whole
    // transaction in the db.
    db.rooms.insert_room_info(&info)?;

    usernames
        .into_iter()
        .filter(|username| username != our_username)
        .map(|username| db.rooms.add_user_to_room(&username, room_id_str))
        .collect() //Make use of collect impl on Result.
}
