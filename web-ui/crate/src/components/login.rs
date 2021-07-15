use crate::api;
use crate::error::UiError;
use crate::state::{Action, WebUiDispatcher};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use web_sys::FocusEvent;
use yew::prelude::*;
use yewdux::dispatch::Dispatcher;
use yewdux::prelude::*;
use yewtil::NeqAssign;

#[doc(hidden)]
pub(crate) struct YewduxLogin {
    dispatch: Rc<WebUiDispatcher>,
    link: ComponentLink<YewduxLogin>,
    username: String,
    password: String,
}

pub enum LoginAction {
    UpdateUsername(String),
    UpdatePassword(String),
    Login,
    Noop,
}

pub(crate) type Login = WithDispatch<YewduxLogin>;

async fn do_login(
    dispatch: &WebUiDispatcher,
    username: &str,
    password: &str,
) -> Result<(), UiError> {
    let jwt = api::auth::login(username, password).await?;
    dispatch.send(Action::Login(jwt));
    Ok(())
}

impl Component for YewduxLogin {
    type Message = LoginAction;
    type Properties = WebUiDispatcher;

    fn create(dispatch: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            dispatch: Rc::new(dispatch),
            link,
            username: "".to_string(),
            password: "".to_string(),
        }
    }

    fn update(&mut self, action: Self::Message) -> ShouldRender {
        match action {
            LoginAction::UpdateUsername(username) => self.username = username,
            LoginAction::UpdatePassword(password) => self.password = password,
            LoginAction::Login => {
                let dispatch = self.dispatch.clone();
                let username = self.username.clone();
                let password = self.password.clone();

                spawn_local(async move {
                    do_login(&*dispatch, &username, &password).await;
                });
            }
            _ => (),
        };

        false
    }

    fn change(&mut self, dispatch: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(Rc::new(dispatch))
    }

    fn view(&self) -> Html {
        let do_the_login = self.link.callback(move |e: FocusEvent| {
            e.prevent_default();
            LoginAction::Login
        });

        let update_username = self
            .link
            .callback(|e: InputData| LoginAction::UpdateUsername(e.value));

        let update_password = self
            .link
            .callback(|e: InputData| LoginAction::UpdatePassword(e.value));

        html! {
            <div>
                <form onsubmit=do_the_login>
                <label for="username">{"Username:"}</label>
                <input oninput=update_username id="username" name="username" type="text" placeholder="Username" />
                <label for="password">{"Password:"}</label>
                <input oninput=update_password id="password" name="password" type="password" placeholder="Password" />
                <input type="submit" value="Log In" />
                </form>
                //<button onclick=refresh_jwt>{ "Refresh JWT" }</button>
                <div>
            { "Current JWT: " }
            { self.dispatch.state().jwt_token.as_ref().unwrap_or(&"[not set]".to_string()) }
            </div>
                </div>
        }
    }

    fn destroy(&mut self) {}
}
