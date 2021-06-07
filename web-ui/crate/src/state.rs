use yewdux::prelude::*;

#[derive(Clone)]
pub(crate) struct Room {
    pub room_id: String,
    pub display_name: String,
}

#[derive(Default, Clone)]
pub(crate) struct WebUiState {
    pub rooms: Vec<Room>,
}

pub(crate) enum Action {
    AddRoom(Room),
}

impl Reducer for WebUiState {
    type Action = Action;

    fn new() -> Self {
        Self { rooms: vec![] }
    }

    fn reduce(&mut self, action: Self::Action) -> bool {
        match action {
            Action::AddRoom(room) => {
                self.rooms.push(room.clone());
                true
            }
        }
    }
}

pub(crate) type WebUiDispatcher = DispatchProps<ReducerStore<WebUiState>>;
