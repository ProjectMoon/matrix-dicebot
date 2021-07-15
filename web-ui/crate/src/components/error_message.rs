use crate::api;
use crate::error::UiError;
use crate::state::{Action, WebUiDispatcher};
use std::rc::Rc;
use wasm_bindgen::{prelude::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::FocusEvent;
use yew::prelude::*;
use yewdux::dispatch::Dispatcher;
use yewdux::prelude::*;
use yewtil::NeqAssign;

#[doc(hidden)]
pub(crate) struct YewduxErrorMessage {
    dispatch: WebUiDispatcher,
    link: ComponentLink<YewduxErrorMessage>,
}

pub(crate) type ErrorMessage = WithDispatch<YewduxErrorMessage>;

impl YewduxErrorMessage {
    fn view_error(&self, error: &str) -> Html {
        html! {
            <div class="alert alert-danger" role="alert">
                { error }
            </div>
        }
    }
}

impl Component for YewduxErrorMessage {
    type Message = ();
    type Properties = WebUiDispatcher;

    fn create(dispatch: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { dispatch, link }
    }

    fn update(&mut self, action: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(dispatch)
    }

    fn rendered(&mut self, _first_render: bool) {}

    fn view(&self) -> Html {
        html! {
            <div>
            {
                for self.dispatch.state().error_messages.iter().map(|error| {
                    self.view_error(error)
                })
            }
            </div>
        }
    }

    fn destroy(&mut self) {}
}
