use std::{
    mem,
    sync::{Arc, Mutex},
};

use mapp::provider::Res;
use tauri::{App, Wry};

type GuiBuilder = tauri::Builder<tauri::Wry>;
type SetupHook<R> = Box<dyn FnOnce(&mut App<R>) -> Result<(), Box<dyn std::error::Error>> + Send>;

pub struct Builder {
    inner: Mutex<GuiBuilder>,
    setup_with_app_callback: Arc<Mutex<Vec<SetupHook<Wry>>>>,
}

impl Builder {
    pub async fn new() -> Result<Res<Self>, anyhow::Error> {
        let setup_with_app_callback: Arc<Mutex<Vec<SetupHook<Wry>>>> =
            Arc::new(Mutex::new(Vec::new()));
        let builder = {
            let setup_with_app_callback = setup_with_app_callback.clone();
            GuiBuilder::default().setup(move |app| {
                let mut callbacks = setup_with_app_callback.lock().unwrap();
                for cb in callbacks.drain(..) {
                    cb(app)?;
                }
                Ok(())
            })
        };
        Ok(Res::new(Self {
            inner: Mutex::new(builder),
            setup_with_app_callback,
        }))
    }

    pub fn setup<F>(&self, f: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(GuiBuilder) -> Result<GuiBuilder, anyhow::Error>,
    {
        self.replace(f(self.take())?);
        Ok(())
    }

    pub fn setup_with_app<F>(&self, f: F) -> &Self
    where
        F: FnOnce(&mut App<Wry>) -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
    {
        self.setup_with_app_callback
            .lock()
            .unwrap()
            .push(Box::new(f));
        self
    }

    pub fn take(&self) -> GuiBuilder {
        self.replace(GuiBuilder::default())
    }

    fn replace(&self, builder: GuiBuilder) -> GuiBuilder {
        mem::replace(&mut self.inner.lock().unwrap(), builder)
    }
}
