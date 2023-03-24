use tracing::debug;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{keybinding::{Keybinging, self}, route};

#[derive(Clone, PartialEq)]
pub struct AppContext {
    pub keybinding: Keybinging,
}

pub struct App {
    app_ctx: AppContext,
}

#[derive(Properties, PartialEq)]
pub struct AppProps {
    path: String,
}

impl Component for App {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        debug!("App()");
        Self {
            app_ctx: AppContext {
                keybinding: keybinding::setup(),
            },
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <ContextProvider<AppContext> context={self.app_ctx.clone()}>
                <Switch<route::Route> render={ route::switch }/>
            </ContextProvider<AppContext>>
        }
    }
}
