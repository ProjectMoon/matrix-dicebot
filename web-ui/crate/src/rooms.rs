use crate::api;
use crate::error::UiError;
use crate::state::{Action, Room, WebUiDispatcher};
use std::sync::Arc;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use yew::prelude::*;
use yewdux::dispatch::Dispatcher;
use yewdux::prelude::*;
use yewtil::NeqAssign;

#[doc(hidden)]
pub(crate) struct YewduxRoomList {
    dispatch: WebUiDispatcher,
    link: ComponentLink<YewduxRoomList>,
}

pub(crate) type RoomList = WithDispatch<YewduxRoomList>;

fn view_room(room: &Room) -> Html {
    html! {
       <li>
           {&room.display_name} {" ("}{&room.room_id}{")"}
       </li>
    }
}

async fn load_rooms(dispatch: &WebUiDispatcher) -> Result<(), UiError> {
    let jwt_token = dispatch
        .state()
        .jwt_token
        .as_ref()
        .ok_or(UiError::NotLoggedIn)?;

    let rooms = api::dicebot::rooms_for_user(jwt_token, "@projectmoon:agnos.is").await?;

    for room in rooms {
        dispatch.send(Action::AddRoom(Room {
            room_id: room.room_id,
            display_name: room.display_name,
        }));
    }

    Ok(())
}

async fn do_jwt_stuff(dispatch: &WebUiDispatcher) -> Result<(), UiError> {
    let jwt = api::auth::fetch_jwt().await?;
    dispatch.send(Action::UpdateJwt(jwt));
    Ok(())
}

impl Component for YewduxRoomList {
    type Message = ();
    type Properties = WebUiDispatcher;

    fn create(dispatch: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { dispatch, link }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(dispatch)
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let dispatch = Arc::new(self.dispatch.clone());

            spawn_local(async move {
                //TODO make macro to report errors in some common way:
                //handle_errors!(do_things(&*dispatch).await)
                match load_rooms(&*dispatch).await {
                    Err(e) => console::log_1(&format!("Error: {:?}", e).into()),
                    _ => (),
                }
            });
        }
    }

    fn view(&self) -> Html {
        let dispatch = Arc::new(self.dispatch.clone());
        let dispatch2 = dispatch.clone();

        let the_future = self.link.callback(move |_| {
            let dispatch = dispatch.clone();

            spawn_local(async move {
                //TODO make macro to report errors in some common way:
                //handle_errors!(do_things(&*dispatch).await)
                match load_rooms(&*dispatch).await {
                    Err(e) => console::log_1(&format!("Error: {:?}", e).into()),
                    _ => (),
                }
            });
        });

        let jwt_update = self.link.callback(move |_| {
            let dispatch = dispatch2.clone();
            spawn_local(async move {
                do_jwt_stuff(&*dispatch).await;
            });
        });

        html! {
            <div>
                <button onclick=the_future>{ "Add Room" }</button>
                <button onclick=jwt_update>{ "Fetch JWT" }</button>
                <div> { "Current JWT: " } { self.dispatch.state().jwt_token.as_ref().unwrap_or(&"[not set]".to_string()) }</div>
             <ul>
                {
                    for self.dispatch.state().rooms.iter().map(|room| {
                        view_room(room)
                    })
                }
            </ul>
                </div>
        }
    }
}

//New oath form

//Edit oath
