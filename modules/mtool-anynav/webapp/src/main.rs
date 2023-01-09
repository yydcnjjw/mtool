use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

mod tauri {
    use serde::{de::DeserializeOwned, Serialize};
    use serde_wasm_bindgen::{from_value, to_value};

    pub async fn invoke<Args, Output>(cmd: &str, args: &Args) -> Output
    where
        Args: Serialize + ?Sized,
        Output: DeserializeOwned,
    {
        from_value(crate::invoke(cmd, to_value(args).unwrap()).await).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
struct SearchArgs {
    input: String,
}

#[function_component]
fn App() -> Html {
    let input_node = use_node_ref();
    let waiting_search = use_state(bool::default);

    let oninput = {
        let input_node = input_node.clone();
        let waiting_search = waiting_search.clone();

        Callback::from(move |_| {
            let input = input_node.cast::<HtmlInputElement>().unwrap();

            if !*waiting_search {
                waiting_search.set(true);

                let waiting_search = waiting_search.clone();
                spawn_local(async move {
                    let _: () = tauri::invoke(
                        "search",
                        &SearchArgs {
                            input: input.value(),
                        },
                    )
                    .await;

                    waiting_search.set(false);
                });
            }
        })
    };

    html! {
        <div>
            <input ref={input_node}
                {oninput}
                type="text"
            id="search"/>
            if { *waiting_search } {
                <p>{ "Waiting search" }</p>
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
