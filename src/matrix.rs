use log::error;
use matrix_sdk::{events::room::message::NoticeMessageEventContent, RoomMember};
use matrix_sdk::{
    events::room::message::{InReplyTo, Relation},
    events::room::message::{MessageEventContent, MessageType},
    events::AnyMessageEventContent,
    identifiers::EventId,
    Error as MatrixError,
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

        members
            .into_iter()
            .map(|member| member.user_id().to_string())
            .collect()
    } else {
        vec![]
    }
}

pub async fn send_message(
    client: &Client,
    room_id: &RoomId,
    message: &str,
    reply_to: Option<EventId>,
) {
    let room = match client.get_joined_room(room_id) {
        Some(room) => room,
        _ => return,
    };

    let plain = html2text::from_read(message.as_bytes(), message.len());
    let mut content = MessageEventContent::new(MessageType::Notice(
        NoticeMessageEventContent::html(plain.trim(), message),
    ));

    content.relates_to = reply_to.map(|event_id| Relation::Reply {
        in_reply_to: InReplyTo { event_id },
    });

    let content = AnyMessageEventContent::RoomMessage(content);

    let result = room.send(content, None).await;

    if let Err(e) = result {
        let message = extract_error_message(e);
        error!("Error sending message: {}", message);
    };
}
