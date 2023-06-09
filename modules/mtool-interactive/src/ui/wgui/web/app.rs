use yew::prelude::*;
use yew_router::prelude::*;

use super::route;

pub struct App {}

#[derive(Properties, PartialEq)]
pub struct AppProps {
    path: String,
}

impl Component for App {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <Switch<route::Route> render={ route::switch }/>
        }
    }
}
