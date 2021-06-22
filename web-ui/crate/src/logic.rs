use crate::{
    api,
    state::{Action, Claims, Room, WebUiDispatcher},
};
use jsonwebtoken::{
    dangerous_insecure_decode_with_validation as decode_without_verify, Validation,
};

pub(crate) type LogicResult = Result<Vec<Action>, Action>;

trait LogicResultExt {
    /// Consumes self into the vec of Actions to apply to state,
    /// either the list of successful actions, or a list containing
    /// the error action.
    fn actions(self) -> Vec<Action>;
}

impl LogicResultExt for LogicResult {
    fn actions(self) -> Vec<Action> {
        self.unwrap_or_else(|err_action| vec![err_action])
    }
}

fn map_to_vec(action: Option<Action>) -> Vec<Action> {
    action.map(|a| vec![a]).unwrap_or_default()
}

async fn refresh_ensured_jwt() -> Result<(String, Option<Action>), Action> {
    api::auth::refresh_jwt()
        .await
        .map(|new_jwt| (new_jwt.clone(), Some(Action::UpdateJwt(new_jwt))))
        .map_err(|_| Action::Logout)
}

async fn ensure_jwt(dispatch: &WebUiDispatcher) -> Result<(String, Option<Action>), Action> {
    //TODO lots of clones here. can we avoid?
    use jsonwebtoken::errors::ErrorKind;
    let token = dispatch.state().jwt_token.as_deref().unwrap_or_default();
    let validation: Result<Claims, _> =
        decode_without_verify(token, &Validation::default()).map(|data| data.claims);

    //If valid, simply return token. If expired, attempt to refresh.
    //Otherwise, bubble error.
    let token_and_action = match validation {
        Ok(_) => (token.to_owned(), None),
        Err(e) if matches!(e.kind(), ErrorKind::ExpiredSignature) => refresh_ensured_jwt().await?,
        Err(_) => return Err(Action::Logout), //TODO carry error inside Logout?
    };

    Ok(token_and_action)
}

pub(crate) async fn fetch_rooms(dispatch: &WebUiDispatcher) -> LogicResult {
    let (jwt, jwt_update) = ensure_jwt(dispatch)
        .await
        .map(|(token, update)| (token, map_to_vec(update)))?;

    let rooms: Vec<Action> = api::dicebot::rooms_for_user(&jwt, &dispatch.state().username)
        .await
        .map_err(|e| Action::ErrorMessage(e))?
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
