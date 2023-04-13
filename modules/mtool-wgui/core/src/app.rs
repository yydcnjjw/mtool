use tracing::debug;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{route::Route, switch::ListenSwitch};

pub struct App {}

impl Component for App {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        debug!("App()");
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
              <ListenSwitch<Route> render={switch} />
            </BrowserRouter>
        }
    }
}

fn switch(routes: Route) -> Html {
    routes.call()
}
