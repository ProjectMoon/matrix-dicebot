use oaths::OathsList;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::{components::RouterAnchor, prelude::*};

pub mod grpc;
pub mod oaths;

#[derive(Switch, Clone, Debug)]
pub enum AppRoute {
    #[to = "/oaths"]
    Oaths,
    #[to = "/commitments"]
    Commitments,
    #[to = "/studies"]
    Studies,
    #[to = "/divination"]
    RunicDivination,
    #[to = "/"]
    Index,
}

type AppRouter = Router<AppRoute>;
type AppAnchor = RouterAnchor<AppRoute>;

fn render_route(switch: AppRoute) -> Html {
    match switch {
        AppRoute::Oaths => {
            html! {
                <OathsList />
            }
        }
        AppRoute::Commitments => {
            html! {
                <div>{"This is the commitments page."}</div>
            }
        }
        AppRoute::Studies => {
            html! {
                <div>{"This is the studies page."}</div>
            }
        }
        AppRoute::RunicDivination => {
            html! {
                <div>{"This is the runic divination page."}</div>
            }
        }
        AppRoute::Index => {
            html! {
                <div>{"This is the index."}</div>
            }
        }
    }
}

struct AppMenu;

impl Component for AppMenu {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
             <ul>
                <li>
                    <AppAnchor route=AppRoute::Index>{"Home"}</AppAnchor>
                </li>
                <li>
                    <AppAnchor route=AppRoute::Oaths>{"Oaths"}</AppAnchor>
                </li>
                <li>
                    <AppAnchor route=AppRoute::Commitments>{"Commitments"}</AppAnchor>
                </li>
                <li>
                    <AppAnchor route=AppRoute::Studies>{"Studies"}</AppAnchor>
                </li>
                <li>
                    <AppAnchor route=AppRoute::RunicDivination>{"Runic Divination"}</AppAnchor>
                </li>
             </ul>
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
            {"Hello World"}
            <AppMenu />
            <AppRouter render=AppRouter::render(render_route) />
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    //App::<EncryptedImage>::new().mount_with_props(body, props);
    yew::start_app::<App>();
}
