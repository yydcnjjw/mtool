use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::*;

use super::{keybinding::Keybinging, route::Route, switch::ListenSwitch};

#[derive(Clone, PartialEq)]
pub struct AppContext {
    pub keybinding: Keybinging,
}

pub struct App {
    ctx: AppContext,
}

impl Component for App {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let keybinding = Keybinging::new();

        keybinding.setup_on_keydown(|f| {
            window().unwrap().set_onkeydown(f);
        });

        Self {
            ctx: AppContext { keybinding },
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <ContextProvider<AppContext> context={self.ctx.clone()}>
                  <ListenSwitch<Route> render={switch} />
                </ContextProvider<AppContext>>
            </BrowserRouter>
        }
    }
}

fn switch(routes: Route) -> Html {
    routes.call()
}
