use crate::components::error_message::ErrorMessage;
use crate::components::login::Login;
use error::UiError;
use rooms::RoomList;
use rooms::YewduxRoomList;
use state::{Action, AuthState, WebUiDispatcher};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::{components::RouterAnchor, prelude::*, switch::Permissive};
use yewdux::prelude::*;
use yewtil::NeqAssign;

pub mod api;
pub mod components;
pub mod error;
pub mod grpc;
pub mod rooms;
pub mod state;

#[derive(Clone, Debug, Switch)]
pub enum AppRoute {
    #[to = "/rooms"]
    Rooms,
    #[to = "/rooms/{room_id}"]
    Room { room_id: String },
    #[to = "/"]
    Index,
}

type AppRouter = Router<AppRoute>;
type AppAnchor = RouterAnchor<AppRoute>; //For rendering clickable links.

fn render_route(routes: AppRoute) -> Html {
    match routes {
        AppRoute::Rooms => {
            html! {
                <RoomList />
            }
        }
        AppRoute::Room { room_id } => {
            html! {
                <div>{"This is the specific room page."}</div>
            }
        }
        AppRoute::Index => {
            html! {
                <div>
                    <ErrorMessage />
                    <RoomList />
                </div>
            }
        }
    }
}

struct YewduxApp {
    link: ComponentLink<YewduxApp>,
    dispatch: WebUiDispatcher,
}

type App = WithDispatch<YewduxApp>;

async fn refresh_jwt(dispatch: &WebUiDispatcher) {
    match api::auth::refresh_jwt().await {
        Ok(jwt) => {
            dispatch.send(Action::Login(jwt));
        }
        Err(e) => {
            web_sys::console::log_1(&e.to_string().into());
            dispatch.send(Action::ChangeAuthState(AuthState::NotLoggedIn));
        }
    }
}

impl Component for YewduxApp {
    type Message = ();
    type Properties = WebUiDispatcher;

    fn create(dispatch: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { dispatch, link }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(dispatch)
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let auth_state = self.dispatch.state().auth_state;

            if auth_state == AuthState::RefreshNeeded {
                let dispatch = self.dispatch.clone();
                spawn_local(async move {
                    refresh_jwt(&dispatch).await;
                });
            }
        }
    }

    fn view(&self) -> Html {
        let auth_state = self.dispatch.state().auth_state;

        match auth_state {
            AuthState::RefreshNeeded => {
                html! {
                    <div>{"Loading..."}</div>
                }
            }
            AuthState::NotLoggedIn => {
                html! {
                    <Login />
                }
            }
            AuthState::LoggedIn => {
                html! {
                    <div>
                        <div class="alert alert-primary" role="alert">
                            {"Hello World"}
                        </div>
                        <div>
                            <AppRouter render=AppRouter::render(render_route) />
                        </div>
                    </div>
                }
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::start_app::<App>();
}
