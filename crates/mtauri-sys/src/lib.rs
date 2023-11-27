pub mod event;
pub mod window;

use std::fmt;

use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;

mod ffi {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["__TAURI_INTERNALS__"], catch)]
        pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
    }
}

fn to_anyhow<E>(e: E) -> anyhow::Error
where
    E: fmt::Display,
{
    anyhow::anyhow!("{}", e)
}

pub async fn invoke_raw<Output>(cmd: &str, args: JsValue) -> Result<Output, anyhow::Error>
where
    Output: DeserializeOwned,
{
    match ffi::invoke(cmd, args).await {
        Ok(v) => Ok(from_value(v).map_err(to_anyhow)?),
        Err(e) => Err(anyhow::Error::from(
            from_value::<serde_error::Error>(e).map_err(to_anyhow)?,
        )),
    }
}

pub async fn invoke<Args, Output>(cmd: &str, args: &Args) -> Result<Output, anyhow::Error>
where
    Args: Serialize + ?Sized,
    Output: DeserializeOwned,
{
    invoke_raw(cmd, to_value(args).map_err(to_anyhow)?).await
}
