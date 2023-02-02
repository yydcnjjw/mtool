use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;

mod ffi {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"], catch)]
        pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
    }
}

pub async fn invoke<Args, Output>(cmd: &str, args: &Args) -> Result<Output, JsValue>
where
    Args: Serialize + ?Sized,
    Output: DeserializeOwned,
{
    Ok(from_value(ffi::invoke(cmd, to_value(args)?).await?)?)
}

pub use event::Event;

pub mod event {
    use serde::{de::DeserializeOwned, Deserialize};
    use serde_wasm_bindgen::from_value;
    use wasm_bindgen::{prelude::Closure, JsValue};

    mod ffi {
        use wasm_bindgen::prelude::*;
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], catch)]
            pub async fn listen(
                event: &str,
                handler: &Closure<dyn FnMut(JsValue) -> Result<(), JsValue>>,
            ) -> Result<JsValue, JsValue>;
        }
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Event<T> {
        /// Event name
        pub event: String,
        /// Event identifier used to unlisten
        pub id: f32,
        /// Event payload
        pub payload: T,
        /// The label of the window that emitted this event
        pub window_label: Option<String>,
    }

    pub async fn listen<Handler, T>(
        event: &str,
        mut handler: Handler,
    ) -> Result<impl Fn() -> Result<(), JsValue>, JsValue>
    where
        Handler: FnMut(Event<T>) -> Result<(), JsValue> + 'static,
        T: DeserializeOwned + 'static,
    {
        let closure = Closure::new(move |raw| handler(from_value::<Event<T>>(raw)?));

        let unlisten = ffi::listen(event, &closure).await;

        closure.forget();

        unlisten.map(|v| {
            let v = js_sys::Function::from(v);
            move || {
                v.call0(&JsValue::NULL)?;
                Ok(())
            }
        })
    }
}

pub mod window {
    mod ffi {
        use wasm_bindgen::prelude::*;
        #[wasm_bindgen]
        extern "C" {

            pub type PhysicalSize;
            #[wasm_bindgen(constructor, js_namespace = ["window", "__TAURI__", "window"])]
            pub fn new(width: usize, height: usize) -> PhysicalSize;

            #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "window", "appWindow"], catch)]
            pub async fn setSize(size: PhysicalSize) -> Result<(), JsValue>;
        }
    }

    #[derive(Debug)]
    pub struct PhysicalSize {
        pub width: usize,
        pub height: usize,
    }

    pub async fn set_size(size: PhysicalSize) {
        ffi::setSize(ffi::PhysicalSize::new(size.width, size.height))
            .await
            .unwrap();
    }
}
