use tracing::debug;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::component::{completion::Completion, output::Output};

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum Route {
    #[at("/interactive")]
    Home,
    #[at("/interactive/completion/:id")]
    Completion { id: String },
    #[at("/interactive/output/:id")]
    Output { id: String },
}

pub fn switch(routes: Route) -> Html {
    debug!("{}", routes.to_path());
    match routes {
        Route::Home => html! { <h1>{ "Mtool Interactive" }</h1> },
        Route::Completion { id } => html! { <Completion id={id} /> },
        Route::Output { id } => html! { <Output id={id} /> },
    }
}
