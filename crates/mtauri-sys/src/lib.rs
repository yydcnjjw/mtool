pub mod window;
pub mod event;

use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;

mod ffi {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__"], catch)]
        pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
    }
}

pub async fn invoke_raw<Output>(cmd: &str, args: JsValue) -> Result<Output, JsValue>
where
    Output: DeserializeOwned,
{
    Ok(from_value(ffi::invoke(cmd, args).await?)?)
}

pub async fn invoke<Args, Output>(cmd: &str, args: &Args) -> Result<Output, JsValue>
where
    Args: Serialize + ?Sized,
    Output: DeserializeOwned,
{
    Ok(from_value(ffi::invoke(cmd, to_value(args)?).await?)?)
}

