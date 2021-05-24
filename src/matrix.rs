use futures::stream::{self, StreamExt, TryStreamExt};
use log::error;
use matrix_sdk::{events::room::message::NoticeMessageEventContent, room::Joined, StoreError};
use matrix_sdk::{
    events::room::message::{InReplyTo, Relation},
    events::room::message::{MessageEventContent, MessageType},
    events::AnyMessageEventContent,
    identifiers::EventId,
    Error as MatrixError,
};
use matrix_sdk::{identifiers::RoomId, identifiers::UserId, Client};

/// Extracts more detailed error messages out of a matrix SDK error.
fn extract_error_message(error: MatrixError) -> String {
    use matrix_sdk::{Error::Http, HttpError};
    if let Http(HttpError::Api(ruma_err)) = error {
        ruma_err.to_string()
    } else {
        error.to_string()
    }
}

/// Retrieve a list of users in a given room.
pub async fn get_users_in_room(
    client: &Client,
    room_id: &RoomId,
) -> Result<Vec<String>, MatrixError> {
    if let Some(joined_room) = client.get_joined_room(room_id) {
        let members = joined_room.joined_members().await?;

        Ok(members
            .into_iter()
            .map(|member| member.user_id().to_string())
            .collect())
    } else {
        Ok(vec![])
    }
}

pub async fn get_rooms_for_user(
    client: &Client,
    user: &UserId,
) -> Result<Vec<Joined>, MatrixError> {
    // Carries errors through, in case we cannot load joined user IDs
    // from the room for some reason.
    let user_is_in_room = |room: Joined| async move {
        match room.joined_user_ids().await {
            Ok(users) => match users.contains(user) {
                true => Some(Ok(room)),
                false => None,
            },
            Err(e) => Some(Err(e)),
        }
    };

    let rooms_for_user: Vec<Joined> = stream::iter(client.joined_rooms())
        .filter_map(user_is_in_room)
        .try_collect()
        .await?;

    Ok(rooms_for_user)
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
        in_reply_to: InReplyTo::new(event_id),
    });

    let content = AnyMessageEventContent::RoomMessage(content);

    let result = room.send(content, None).await;

    if let Err(e) = result {
        let message = extract_error_message(e);
        error!("Error sending message: {}", message);
    };
}
