use crate::{
    api,
    error::UiError,
    state::{Action, Room, WebUiDispatcher},
};
use jsonwebtoken::{
    dangerous_insecure_decode_with_validation as decode_without_verify, Validation,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize,
    sub: String,
}

fn map_to_vec(action: Option<Action>) -> Vec<Action> {
    action.map(|a| vec![a]).unwrap_or_default()
}

async fn ensure_jwt(dispatch: &WebUiDispatcher) -> Result<(String, Option<Action>), UiError> {
    //TODO we should add a logout action and return it from here if there's an error when refreshing.
    //TODO somehow have to handle actions on an error!

    //TODO lots of clones here. can we avoid?
    use jsonwebtoken::errors::ErrorKind;
    let token = dispatch.state().jwt_token.as_deref().unwrap_or_default();
    let validation =
        decode_without_verify::<Claims>(token, &Validation::default()).map(|data| data.claims);

    //If valid, simply return token. If expired, attempt to refresh.
    //Otherwise, bubble error.
    let token_and_action = match validation {
        Ok(_) => (token.to_owned(), None),
        Err(e) if matches!(e.kind(), ErrorKind::ExpiredSignature) => {
            match api::auth::refresh_jwt().await {
                Ok(new_jwt) => (new_jwt.clone(), Some(Action::UpdateJwt(new_jwt))),
                Err(e) => return Err(e.into()), //TODO logout action
            }
        }
        Err(e) => return Err(e.into()),
    };

    Ok(token_and_action)
}

pub(crate) async fn fetch_rooms(dispatch: &WebUiDispatcher) -> Result<Vec<Action>, UiError> {
    let (jwt, jwt_update) = ensure_jwt(dispatch)
        .await
        .map(|(token, update)| (token, map_to_vec(update)))?;

    //Use new JWT to list rooms from graphql.
    //TODO get username from state.
    let rooms: Vec<Action> = api::dicebot::rooms_for_user(&jwt, "@projectmoon:agnos.is")
        .await?
        .into_iter()
        .map(|room| {
            Action::AddRoom(Room {
                room_id: room.room_id,
                display_name: room.display_name,
            })
        })
        .collect();

    Ok(rooms.into_iter().chain(jwt_update).collect())
}
