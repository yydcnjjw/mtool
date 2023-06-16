use mapp::provider::Res;
use yew::prelude::*;
use yew_router::prelude::*;

use super::{keybinding::Keybinding, route::Route, switch::ListenSwitch, template::Templator};

#[derive(Clone, PartialEq, Properties)]
pub struct AppContext {
    pub keybinding: Keybinding,
    pub templator: Res<Templator>,
}

pub struct App {}

impl Component for App {
    type Message = ();

    type Properties = AppContext;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <ContextProvider<AppContext> context={ ctx.props().clone() }>
                  <ListenSwitch<Route> render={ switch } />
                </ContextProvider<AppContext>>
            </BrowserRouter>
        }
    }
}

fn switch(routes: Route) -> Html {
    routes.call()
}
