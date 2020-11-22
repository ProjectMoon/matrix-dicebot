use matrix_sdk::{identifiers::RoomId, Client, Room};

/// Retrieve a list of users in a given room.
pub async fn get_users_in_room(client: &Client, room_id: &RoomId) -> Vec<String> {
    if let Some(joined_room) = client.get_joined_room(room_id).await {
        let joined_room: Room = joined_room.read().await.clone();
        joined_room
            .joined_members
            .keys()
            .map(|user_id| format!("@{}:{}", user_id.localpart(), user_id.server_name()))
            .collect()
    } else {
        vec![]
    }
}
