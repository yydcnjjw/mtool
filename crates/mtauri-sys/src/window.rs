use serde::de::DeserializeOwned;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::{
    prelude::Closure,
    JsValue,
};

use crate::event::Event;

mod ffi {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "window"])]
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

        #[wasm_bindgen(method, catch)]
        pub async fn setSize(this: &WebviewWindow, size: JsValue) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn setPosition(this: &WebviewWindow, pos: JsValue) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn listen(
            this: &WebviewWindow,
            event: &str,
            handler: &Closure<dyn FnMut(JsValue) -> Result<(), JsValue>>,
        ) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch)]
        pub fn getCurrent() -> Result<WebviewWindow, JsValue>;

    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Physical { width: usize, height: usize },
    Logical { width: usize, height: usize },
}

impl Size {
    pub fn new_physical(width: usize, height: usize) -> Self {
        Self::Physical { width, height }
    }

    pub fn new_logical(width: usize, height: usize) -> Self {
        Self::Physical { width, height }
    }

    pub fn get(&self) -> (usize, usize) {
        match self {
            Size::Physical { width, height } => (*width, *height),
            Size::Logical { width, height } => (*width, *height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Physical { x: usize, y: usize },
    Logical { x: usize, y: usize },
}

impl Position {
    pub fn new_physical(x: usize, y: usize) -> Self {
        Self::Physical { x, y }
    }

    pub fn new_logical(x: usize, y: usize) -> Self {
        Self::Physical { x, y }
    }

    pub fn get(&self) -> (usize, usize) {
        match self {
            Position::Physical { x, y } => (*x, *y),
            Position::Logical { x, y } => (*x, *y),
        }
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

    pub async fn set_size(&self, size: Size) -> Result<(), JsValue> {
        self.handle
            .setSize(match size {
                Size::Physical { width, height } => ffi::PhysicalSize::new(width, height).into(),
                Size::Logical { width, height } => ffi::LogicalSize::new(width, height).into(),
            })
            .await
    }

    pub async fn set_position(&self, pos: Position) -> Result<(), JsValue> {
        self.handle
            .setPosition(match pos {
                Position::Physical { x, y } => ffi::PhysicalPosition::new(x, y).into(),
                Position::Logical { x, y } => ffi::LogicalPosition::new(x, y).into(),
            })
            .await
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
}
