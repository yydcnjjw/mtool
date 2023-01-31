mod component;
mod event;
mod keybinding;
mod tauri;

use component::output::Output;
use gloo_console::log;
use keybinding::Keybinging;
use mkeybinding::KeyCombine;
use msysev::KeyAction;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;
use yew::{platform::spawn_local, prelude::*};
use yew_router::prelude::*;

use crate::{component::completion::Completion, event::into_key_event};

#[derive(Clone, PartialEq)]
pub struct AppContext {
    pub keybinding: Keybinging,
}

#[derive(Clone, Routable, PartialEq, Debug)]
enum Route {
    #[at("/")]
    Home,
    #[at("/completion")]
    Completion,
    #[at("/output")]
    Output,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <h1>{ "mtool" }</h1> },
        Route::Completion => html! { <Completion/> },
        Route::Output => html! { <Output/> },
    }
}

fn keybinding_setup() -> Keybinging {
    let keybinding = Keybinging::new();
    {
        let keybinding = keybinding.clone();
        let a = Closure::<dyn FnMut(_)>::new(move |e: KeyboardEvent| {
            let keyev = into_key_event(e.clone(), KeyAction::Press);
            if keybinding.dispatch(KeyCombine {
                key: keyev.keycode,
                mods: keyev.modifiers,
            }) {
                e.prevent_default();
            }
        });

        window()
            .unwrap()
            .set_onkeydown(Some(a.as_ref().unchecked_ref()));

        a.forget();
    }

    {
        let keybinding = keybinding.clone();
        spawn_local(async move {
            keybinding.run_loop().await;
        });
    }

    keybinding
}

struct App {
    app_ctx: AppContext,
    route_unlisten: Option<UnListen>,
}

enum Msg {
    RegisterRouteUnListen(UnListen),
}

struct UnListen {
    pub unlisten: Box<dyn Fn() -> Result<(), JsValue>>,
}

impl Drop for UnListen {
    fn drop(&mut self) {
        (*self.unlisten)().unwrap();
    }
}

impl App {
    fn listen_route(ctx: &Context<Self>) {
        ctx.link().send_future(async move {
            let unlisten = tauri::listen("route", move |e: tauri::Event<String>| {
                log!("route to {}", &e.payload);
                window().unwrap().location().set_pathname(&e.payload)
            })
            .await
            .unwrap();

            Msg::RegisterRouteUnListen(UnListen {
                unlisten: Box::new(unlisten),
            })
        });
    }
}

impl Component for App {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self::listen_route(ctx);

        Self {
            app_ctx: AppContext {
                keybinding: keybinding_setup(),
            },
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
            <ContextProvider<AppContext> context={self.app_ctx.clone()}>
                <BrowserRouter>
                  <Switch<Route> render={switch} />
                </BrowserRouter>
            </ContextProvider<AppContext>>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
