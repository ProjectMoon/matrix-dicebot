use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::{future_to_promise, spawn_local};
use web_sys::console;
use yew::prelude::*;
use yewdux::prelude::*;
use yewtil::NeqAssign;

#[derive(Clone)]
pub(crate) struct Room {
    room_id: String,
    display_name: String,
}

#[derive(Default, Clone)]
pub(crate) struct RoomListState {
    rooms: Vec<Room>,
}

pub(crate) enum Action {
    AddRoom(Room),
}

impl Reducer for RoomListState {
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

type RoomListDispatch = DispatchProps<ReducerStore<RoomListState>>;

//Oaths list
#[doc(hidden)]
pub(crate) struct YewduxRoomList {
    dispatch: RoomListDispatch,
    link: ComponentLink<YewduxRoomList>,
}

pub(crate) type RoomList = WithDispatch<YewduxRoomList>;

fn view_room(room: &Room) -> Html {
    html! {
       <div>
            <div>{room.room_id.clone()}</div>
            <div>{room.display_name.clone()}</div>
        </div>
    }
}

async fn do_things(dispatch: &RoomListDispatch) {
    let rooms = crate::graphql::rooms_for_user("@projectmoon:agnos.is")
        .await
        .unwrap();

    for room in rooms {
        dispatch.send(Action::AddRoom(Room {
            room_id: room.room_id,
            display_name: room.display_name,
        }));
    }
}

impl Component for YewduxRoomList {
    type Message = ();
    type Properties = RoomListDispatch;

    fn create(dispatch: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { dispatch, link }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(dispatch)
    }

    fn view(&self) -> Html {
        let dispatch = Arc::new(self.dispatch.clone());

        let the_future = self.link.callback(move |_| {
            let dispatch = dispatch.clone();

            spawn_local(async move {
                do_things(&*dispatch).await;
            });
        });

        html! {
            <div>
                <button onclick=the_future>{ "Add Room" }</button>
             <ul>
                {
                    for self.dispatch.state().rooms.iter().map(|oath| {
                        view_room(oath)
                    })
                }
            </ul>
                </div>
        }
    }
}

//New oath form

//Edit oath
