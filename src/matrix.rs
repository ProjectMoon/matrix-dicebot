use log::error;
use matrix_sdk::Error as MatrixError;
use matrix_sdk::{
    events::{
        room::message::{MessageEventContent::Notice, NoticeMessageEventContent},
        AnyMessageEventContent::RoomMessage,
    },
    RoomMember,
};
use matrix_sdk::{identifiers::RoomId, Client};

/// Extracts more detailed error messages out of a matrix SDK error.
fn extract_error_message(error: MatrixError) -> String {
    use matrix_sdk::{Error::Http, HttpError};
    if let Http(HttpError::FromHttpResponse(ruma_err)) = error {
        ruma_err.to_string()
    } else {
        error.to_string()
    }
}

/// Retrieve a list of users in a given room.
pub async fn get_users_in_room(client: &Client, room_id: &RoomId) -> Vec<String> {
    if let Some(joined_room) = client.get_joined_room(room_id) {
        let members: Vec<RoomMember> = joined_room.joined_members().await.ok().unwrap_or_default();

        let to_username = |member: RoomMember| {
            format!(
                "@{}:{}",
                member.user_id().localpart(),
                member.user_id().server_name()
            )
        };

        members.into_iter().map(to_username).collect()
    } else {
        vec![]
    }
}

pub async fn send_message(client: &Client, room_id: &RoomId, message: &str) {
    let plain = html2text::from_read(message.as_bytes(), message.len());
    let response = RoomMessage(Notice(NoticeMessageEventContent::html(
        plain.trim(),
        message,
    )));

    let result = client.room_send(&room_id, response, None).await;
    if let Err(e) = result {
        let message = extract_error_message(e);
        error!("Error sending message: {}", message);
    };
}
