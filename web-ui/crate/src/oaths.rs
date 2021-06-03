use yew::prelude::*;
use yewdux::prelude::*;
use yewtil::NeqAssign;

struct Oaths;

#[derive(Clone)]
struct Oath {
    title: String,
    content: String,
}

#[derive(Default, Clone)]
pub(crate) struct OathState {
    oaths: Vec<Oath>,
}

type OathDispatch = DispatchProps<BasicStore<OathState>>;

//Oaths list
pub(crate) struct StatefulOathsList {
    dispatch: OathDispatch,
}

pub(crate) type OathsList = WithDispatch<StatefulOathsList>;

fn view_oath(oath: &Oath) -> Html {
    html! {
       <div>
            <div>{oath.title.clone()}</div>
            <div>{oath.content.clone()}</div>
        </div>
    }
}

impl Component for StatefulOathsList {
    type Message = ();
    type Properties = OathDispatch;

    fn create(dispatch: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { dispatch }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(dispatch)
    }

    fn view(&self) -> Html {
        let add_oath = self.dispatch.reduce_callback(|s| {
            s.oaths.push(Oath {
                title: "yolo".to_string(),
                content: "nolo".to_string(),
            })
        });

        html! {
            <div>
                <button onclick=add_oath>{ "Add Oath" }</button>
             <ul>
                {
                    for self.dispatch.state().oaths.iter().map(|oath| {
                        view_oath(oath)
                    })
                }
            </ul>
                </div>
        }
    }
}

//New oath form

//Edit oath
