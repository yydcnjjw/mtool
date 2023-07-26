use mapp::provider::Res;
use yew::prelude::*;
use yew_router::prelude::*;

use super::{keybinding::Keybinding, route::Route, switch::ListenSwitch, template::Templator};

#[derive(Clone, PartialEq, Properties)]
pub struct WebAppContext {
    pub keybinding: Keybinding,
    pub templator: Res<Templator>,
}

pub struct WebApp {}

impl Component for WebApp {
    type Message = ();

    type Properties = WebAppContext;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <ContextProvider<WebAppContext> context={ ctx.props().clone() }>
                  <ListenSwitch<Route> render={ switch } />
                </ContextProvider<WebAppContext>>
            </BrowserRouter>
        }
    }
}

fn switch(routes: Route) -> Html {
    routes.call()
}
