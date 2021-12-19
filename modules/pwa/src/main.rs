use wasm_bindgen::closure::Closure;
use web_sys::{console, window, Window};
use yew::prelude::*;

enum Msg {
    AddOne,
}

struct Model {
    value: i64,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { value: 0 }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let link = ctx.link();
        html! {
            <div>
                <button onclick={link.callback(|_| Msg::AddOne)}>{ "+1" }</button>
                <p>{ self.value }</p>
            </div>
        }
    }
}

fn main() {
    let window = window().unwrap();
    // window
    //     .open_with_url_and_target_and_features(
    //         "https://www.google.com",
    //         "test",
    //         "left=100,top=100,width=200,height=100,popup",
    //     )
    //     .unwrap();

    // window.move_to(100, 100).unwrap();
    // window.resize_to(200, 200).unwrap();
    // yew::start_app::<Model>();

    console::log_1(&"test".into());

    let sw = window.navigator().service_worker();
    // sw.register("./sw.js").then2(
    //     &Closure::wrap(Box::new(|_registration| {})),
    //     &Closure::wrap(Box::new(|_err| {
    //         console::log_1(&"test".into());
    //     })),
    // );
}
