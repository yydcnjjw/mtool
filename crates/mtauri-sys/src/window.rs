use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::{prelude::Closure, JsValue};

use crate::{event::Event, invoke};

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
        pub async fn center(this: &WebviewWindow) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn hide(this: &WebviewWindow) -> Result<(), JsValue>;

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
        pub fn getCurrent() -> Result<WebviewWindow, JsValue>;

    }
}

#[derive(Debug, Clone)]
pub struct Window {
    handle: ffi::WebviewWindow,
}

impl Window {
    pub fn current() -> Result<Self, JsValue> {
        Ok(Self {
            handle: ffi::getCurrent()?,
        })
    }

    pub fn new(label: &str) -> Result<Self, JsValue> {
        Ok(Self {
            handle: ffi::WebviewWindow::new(&label)?,
        })
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

        // .setSize(match size {
        //     Size::Physical { width, height } => ffi::PhysicalSize::new(width, height).into(),
        //     Size::Logical { width, height } => ffi::LogicalSize::new(width, height).into(),
        // })
        // .await
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

        // self.handle
        //     .setPosition(match pos {
        //         Position::Physical { x, y } => ffi::PhysicalPosition::new(x, y).into(),
        //         Position::Logical { x, y } => ffi::LogicalPosition::new(x, y).into(),
        //     })
        //     .await
    }

    pub async fn center(&self) -> Result<(), JsValue> {
        self.handle.center().await
    }

    pub async fn hide(&self) -> Result<(), JsValue> {
        self.handle.hide().await
    }

    pub async fn listen<Handler, T>(
        &self,
        event: &str,
        mut handler: Handler,
    ) -> Result<impl Fn() -> Result<(), JsValue>, JsValue>
    where
        Handler: FnMut(Event<T>) -> Result<(), JsValue> + 'static,
        T: DeserializeOwned + 'static,
    {
        let closure = Closure::new(move |raw| handler(from_value::<Event<T>>(raw)?));

        let unlisten = self.handle.listen(event, &closure).await;

        closure.forget();

        unlisten.map(|v| {
            let v = js_sys::Function::from(v);
            move || {
                v.call0(&JsValue::NULL)?;
                Ok(())
            }
        })
    }

    pub async fn emit<T>(&self, event: &str, payload: &T) -> Result<(), anyhow::Error>
    where
        T: Serialize,
    {
        self.handle
            .emit(
                event,
                serde_wasm_bindgen::to_value(payload).map_err(|e| anyhow::anyhow!("{}", e))?,
            )
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))
    }

    // pub async fn emit_to<T>(&self, event: &str, payload: &T) -> Result<(), anyhow::Error>
    // where
    //     T: Serialize,
    // {
    //     self.handle
    //         .emit(
    //             event,
    //             serde_wasm_bindgen::to_value(payload).map_err(|e| anyhow::anyhow!("{}", e))?,
    //         )
    //         .await
    //         .map_err(|e| anyhow::anyhow!("{:?}", e))
    // }

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
                serde_wasm_bindgen::to_value(label).map_err(|e| anyhow::anyhow!("{}", e))?,
                event,
                serde_wasm_bindgen::to_value(payload).map_err(|e| anyhow::anyhow!("{}", e))?,
            )
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))
    }

    pub async fn emit_to_self<T>(&self, event: &str, payload: &T) -> Result<(), anyhow::Error>
    where
        T: Serialize,
    {
        self.emit_to_window(&self.handle.label(), event, payload)
            .await
    }
}
