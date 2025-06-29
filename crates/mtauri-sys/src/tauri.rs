pub use crate::error::IntoAnyhowError;
use mapp::{
    anyhow,
    serde::{de::DeserializeOwned, Serialize},
    serde_wasm_bindgen::{from_value, to_value},
    wasm_bindgen::JsValue,
};

mod ffi {
    use mapp::{
        wasm_bindgen::{self, prelude::*},
        wasm_bindgen_futures,
    };
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["__TAURI_INTERNALS__"], catch)]
        pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
    }
}

pub async fn invoke_raw<Output>(cmd: &str, args: JsValue) -> Result<Output, anyhow::Error>
where
    Output: DeserializeOwned,
{
    from_value(ffi::invoke(cmd, args).await.into_anyhow()?).into_anyhow()
}

pub async fn invoke<Args, Output>(cmd: &str, args: &Args) -> Result<Output, anyhow::Error>
where
    Args: Serialize + ?Sized,
    Output: DeserializeOwned,
{
    invoke_raw(cmd, to_value(args).into_anyhow()?).await
}
