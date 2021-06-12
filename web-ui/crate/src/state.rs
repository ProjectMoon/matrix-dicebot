use crate::error::UiError;
use wasm_bindgen::{prelude::Closure, JsCast};
use yewdux::prelude::*;

#[derive(Clone)]
pub(crate) struct Room {
    pub room_id: String,
    pub display_name: String,
}

#[derive(Default, Clone)]
pub(crate) struct WebUiState {
    pub jwt_token: Option<String>,
    pub rooms: Vec<Room>,
    pub error_messages: Vec<String>,
}

pub(crate) enum Action {
    UpdateJwt(String),
    AddRoom(Room),
    ErrorMessage(UiError),
    ClearErrorMessage,
}

impl Reducer for WebUiState {
    type Action = Action;

    fn new() -> Self {
        Self::default()
    }

    fn reduce(&mut self, action: Self::Action) -> bool {
        match action {
            Action::UpdateJwt(jwt_token) => self.jwt_token = Some(jwt_token),
            Action::AddRoom(room) => self.rooms.push(room.clone()),
            Action::ErrorMessage(error) => self.error_messages.push(error.to_string()),
            Action::ClearErrorMessage => {
                if self.error_messages.len() > 0 {
                    self.error_messages.remove(0);
                }
            }
        };

        true
    }
}

pub(crate) type WebUiDispatcher = DispatchProps<ReducerStore<WebUiState>>;

pub(crate) trait DispatchExt {
    /// Dispatch an error message and then clear it from the state
    /// after a few seconds.
    fn dispatch_error(&self, error: UiError);
}

impl DispatchExt for WebUiDispatcher {
    fn dispatch_error(&self, error: UiError) {
        self.send(Action::ErrorMessage(error));

        // This is a very hacky way to do this. At the very least, we
        // should not leak memory, and preferably there's a cleaner
        // way to actually dispatch the clear action.
        let window = web_sys::window().unwrap();
        let dispatch = self.clone();
        let clear_it = Closure::wrap(
            Box::new(move || dispatch.send(Action::ClearErrorMessage)) as Box<dyn Fn()>
        );

        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                clear_it.as_ref().unchecked_ref(),
                4000,
            )
            .expect("could not add clear error handler.");

        clear_it.forget();
    }
}
