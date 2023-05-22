use serde::{de::DeserializeOwned, Deserialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::{prelude::Closure, JsValue};

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

mod ffi {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    extern "C" {
        #[wasm_bindgen(catch)]
        pub async fn listen(
            event: &str,
            handler: &Closure<dyn FnMut(JsValue) -> Result<(), JsValue>>,
        ) -> Result<JsValue, JsValue>;
    }
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
