mod component;
mod event;
mod keybinding;
mod switch;
mod tauri;

use gloo_console::debug;
use keybinding::Keybinging;
use mkeybinding::KeyCombine;
use msysev::KeyAction;
use switch::ListenSwitch;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;
use yew::{platform::spawn_local, prelude::*};
use yew_router::prelude::*;

use crate::event::into_key_event;

#[derive(Clone, PartialEq)]
pub struct AppContext {
    pub keybinding: Keybinging,
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
}

impl Component for App {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        debug!("App()");
        Self {
            app_ctx: AppContext {
                keybinding: keybinding_setup(),
            },
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <ContextProvider<AppContext> context={self.app_ctx.clone()}>
                <BrowserRouter>
                  <ListenSwitch/>
                </BrowserRouter>
            </ContextProvider<AppContext>>
        }
    }
}

fn main() {
    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     .with_ansi(false) // Only partially supported across browsers
    //     .without_time()
    //     .with_writer(MakeConsoleWriter); // write events to the console
    // let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    // tracing_subscriber::registry()
    //     .with(fmt_layer)
    //     .with(perf_layer)
    //     .init(); // Install these as subscribers to tracing events

    yew::Renderer::<App>::new().render();
}
