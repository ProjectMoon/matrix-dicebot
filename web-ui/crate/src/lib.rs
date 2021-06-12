use crate::components::error_message::ErrorMessage;
use crate::components::login::Login;
use rooms::RoomList;
use rooms::YewduxRoomList;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

pub mod api;
pub mod components;
pub mod error;
pub mod grpc;
pub mod rooms;
pub mod state;

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum AppRoute {
    #[at("/rooms")]
    Rooms,
    #[at("/rooms/{room_id}")]
    Room { room_id: String },
    #[at("/")]
    Index,
}

type AppRouter = Router<AppRoute>;
type AppAnchor = Link<AppRoute>; //For rendering clickable links.

fn render_route(routes: &AppRoute) -> Html {
    match routes {
        AppRoute::Rooms => {
            html! {
                <RoomList />
            }
        }
        AppRoute::Room { room_id } => {
            html! {
                <div>{"This is the specifi roompage."}</div>
            }
        }
        AppRoute::Index => {
            html! {
                <div>
                    <Login />
                    <ErrorMessage />
                    <RoomList />
                </div>
            }
        }
    }
}

struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
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

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::start_app::<App>();
}
