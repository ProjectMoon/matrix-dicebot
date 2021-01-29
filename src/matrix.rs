use matrix_sdk::{identifiers::RoomId, Client};

/// Retrieve a list of users in a given room.
pub async fn get_users_in_room(client: &Client, room_id: &RoomId) -> Vec<String> {
    if let Some(joined_room) = client.get_joined_room(room_id) {
        joined_room
            .joined_members()
            .await
            .ok()
            .unwrap_or_default()
            .into_iter()
            .map(|member| {
                format!(
                    "@{}:{}",
                    member.user_id().localpart(),
                    member.user_id().server_name()
                )
            })
            .collect()
    } else {
        vec![]
    }
}
