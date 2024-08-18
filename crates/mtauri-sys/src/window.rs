use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::{prelude::Closure, JsValue};

use crate::{event::Event, invoke, IntoAnyhowError};

pub use dpi::*;

mod ffi {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(js_namespace = ["__TAURI__", "window"])]
    extern "C" {
        #[derive(Debug, Clone)]
        pub type LogicalSize;
        #[wasm_bindgen(constructor)]
        pub fn new(width: usize, height: usize) -> LogicalSize;

        #[derive(Debug, Clone)]
        pub type PhysicalSize;
        #[wasm_bindgen(constructor)]
        pub fn new(width: usize, height: usize) -> PhysicalSize;

        #[derive(Debug, Clone)]
        pub type LogicalPosition;
        #[wasm_bindgen(constructor)]
        pub fn new(x: usize, y: usize) -> LogicalPosition;

        #[derive(Debug, Clone)]
        pub type PhysicalPosition;
        #[wasm_bindgen(constructor)]
        pub fn new(x: usize, y: usize) -> PhysicalPosition;

        #[derive(Debug, Clone)]
        pub type WebviewWindow;
        #[wasm_bindgen(constructor, catch)]
        pub fn new(label: &str) -> Result<WebviewWindow, JsValue>;

        #[wasm_bindgen(method, getter)]
        pub fn label(this: &WebviewWindow) -> String;

        #[wasm_bindgen(method, catch)]
        pub async fn setSize(this: &WebviewWindow, size: JsValue) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn setPosition(this: &WebviewWindow, pos: JsValue) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn outerPosition(this: &WebviewWindow) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn center(this: &WebviewWindow) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn hide(this: &WebviewWindow) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn setResizable(this: &WebviewWindow, value: JsValue) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn listen(
            this: &WebviewWindow,
            event: &str,
            handler: &Closure<dyn FnMut(JsValue) -> Result<(), JsValue>>,
        ) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn emit(
            this: &WebviewWindow,
            event: &str,
            payload: JsValue,
        ) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn emitTo(
            this: &WebviewWindow,
            target: JsValue,
            event: &str,
            payload: JsValue,
        ) -> Result<(), JsValue>;

        #[wasm_bindgen(catch)]
        pub fn getCurrentWindow() -> Result<WebviewWindow, JsValue>;

    }
}

#[derive(Debug, Clone)]
pub struct Window {
    handle: ffi::WebviewWindow,
}

impl Window {
    pub fn current() -> Result<Self, anyhow::Error> {
        Ok(Self {
            handle: ffi::getCurrentWindow().into_anyhow()?,
        })
    }

    pub fn new(label: &str) -> Result<Self, anyhow::Error> {
        Ok(Self {
            handle: ffi::WebviewWindow::new(&label).into_anyhow()?,
        })
    }

    pub fn label(&self) -> String {
        self.handle.label()
    }

    pub async fn set_size(&self, size: Size) -> Result<(), anyhow::Error> {
        #[derive(Serialize)]
        struct Args {
            label: String,
            value: Size,
        }

        invoke(
            "plugin:window|set_size",
            &Args {
                label: self.handle.label(),
                value: size,
            },
        )
        .await
    }

    pub async fn set_position(&self, pos: Position) -> Result<(), anyhow::Error> {
        #[derive(Serialize)]
        struct Args {
            label: String,
            value: Position,
        }

        invoke(
            "plugin:window|set_position",
            &Args {
                label: self.handle.label(),
                value: pos,
            },
        )
        .await
    }

    pub async fn outer_position(&self) -> Result<PhysicalPosition<i32>, anyhow::Error> {
        Ok(
            serde_wasm_bindgen::from_value(self.handle.outerPosition().await.into_anyhow()?)
                .into_anyhow()?,
        )
    }

    pub async fn center(&self) -> Result<(), anyhow::Error> {
        self.handle.center().await.into_anyhow()
    }

    pub async fn hide(&self) -> Result<(), anyhow::Error> {
        self.handle.hide().await.into_anyhow()
    }

    pub async fn set_resizable(&self, value: bool) -> Result<(), anyhow::Error> {
        self.handle
            .setResizable(JsValue::from_bool(value))
            .await
            .into_anyhow()
    }

    pub async fn listen<Handler, T>(
        &self,
        event: &str,
        mut handler: Handler,
    ) -> Result<impl Fn() -> Result<(), JsValue>, anyhow::Error>
    where
        Handler: FnMut(Event<T>) -> Result<(), JsValue> + 'static,
        T: DeserializeOwned + 'static,
    {
        let closure = Closure::new(move |raw| handler(from_value::<Event<T>>(raw)?));

        let unlisten = self.handle.listen(event, &closure).await;

        closure.forget();

        unlisten
            .map(|v| {
                let v = js_sys::Function::from(v);
                move || {
                    v.call0(&JsValue::NULL)?;
                    Ok(())
                }
            })
            .into_anyhow()
    }

    pub async fn emit<T>(&self, event: &str, payload: &T) -> Result<(), anyhow::Error>
    where
        T: Serialize,
    {
        self.handle
            .emit(event, serde_wasm_bindgen::to_value(payload).into_anyhow()?)
            .await
            .into_anyhow()
    }

    pub async fn emit_to_window<T>(
        &self,
        label: &str,
        event: &str,
        payload: &T,
    ) -> Result<(), anyhow::Error>
    where
        T: Serialize,
    {
        self.handle
            .emitTo(
                serde_wasm_bindgen::to_value(label).into_anyhow()?,
                event,
                serde_wasm_bindgen::to_value(payload).into_anyhow()?,
            )
            .await
            .into_anyhow()
    }

    pub async fn emit_to_self<T>(&self, event: &str, payload: &T) -> Result<(), anyhow::Error>
    where
        T: Serialize,
    {
        self.emit_to_window(&self.handle.label(), event, payload)
            .await
    }
}
