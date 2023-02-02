use gloo_console::debug;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    component::{completion::Completion, output::Output},
    tauri,
};

#[derive(Clone, Routable, PartialEq, Debug)]
enum Route {
    #[at("/")]
    Home,
    #[at("/completion/:id")]
    Completion { id: String },
    #[at("/output/:id")]
    Output { id: String },
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <h1>{ "mtool" }</h1> },
        Route::Completion { id } => html! { <Completion id={id} /> },
        Route::Output { id } => html! { <Output id={id} /> },
    }
}

pub struct ListenSwitch {
    route_unlisten: Option<UnListen>,
}

pub enum Msg {
    RegisterRouteUnListen(UnListen),
}

pub struct UnListen {
    pub unlisten: Box<dyn Fn() -> Result<(), JsValue>>,
}

impl Drop for UnListen {
    fn drop(&mut self) {
        (*self.unlisten)().unwrap();
    }
}

impl ListenSwitch {
    fn listen_route(ctx: &Context<Self>) {
        let link = ctx.link().clone();
        ctx.link().send_future(async move {
            let unlisten = tauri::event::listen("route", move |e: tauri::Event<String>| {
                debug!("try route to ", &e.payload);
                if let Some(nav) = link.navigator() {
                    nav.push(&Route::recognize(&e.payload).unwrap());
                }
                Ok(())
            })
            .await
            .unwrap();

            Msg::RegisterRouteUnListen(UnListen {
                unlisten: Box::new(unlisten),
            })
        });
    }
}

impl Component for ListenSwitch {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self::listen_route(ctx);

        Self {
            route_unlisten: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RegisterRouteUnListen(unlisten) => {
                self.route_unlisten = Some(unlisten);
                false
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <Switch<Route> render={switch} />
        }
    }
}
