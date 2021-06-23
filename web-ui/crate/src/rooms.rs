use crate::api;
use crate::error::UiError;
use crate::logic::{self, LogicResultExt};
use crate::state::{Action, DispatchExt, Room, WebUiDispatcher};
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
    let result = logic::fetch_rooms(dispatch).await;
    let actions = result.actions();

    for action in actions {
        dispatch.send(action);
    }

    Ok(())
}

async fn do_refresh_jwt(dispatch: &WebUiDispatcher) {
    let refresh = api::auth::refresh_jwt().await;

    match refresh {
        Ok(jwt) => dispatch.send(Action::UpdateJwt(jwt)),
        Err(e) => dispatch.dispatch_error(e),
    }
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
        let dispatch3 = dispatch.clone();

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

        let refresh_jwt = self.link.callback(move |_| {
            let dispatch = dispatch3.clone();
            spawn_local(async move { do_refresh_jwt(&*dispatch).await });
        });

        html! {
            <div>
                <button onclick=the_future>{ "Fetch Rooms" }</button>
                <button onclick=refresh_jwt>{ "Refresh JWT" }</button>
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
