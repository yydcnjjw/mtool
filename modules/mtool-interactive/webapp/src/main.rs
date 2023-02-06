mod component;
mod event;
mod keybinding;
mod switch;
mod tauri;

use gloo_console::debug;
use keybinding::Keybinging;
use switch::ListenSwitch;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq)]
pub struct AppContext {
    pub keybinding: Keybinging,
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
                keybinding: keybinding::setup(),
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
