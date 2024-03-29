use std::{
    mem,
    sync::{Arc, Mutex},
};

use mapp::provider::Res;
use tauri::{App, Wry};

type GuiBuilder<R> = tauri::Builder<R>;
type SetupHook<R> = Box<dyn FnOnce(&mut App<R>) -> Result<(), Box<dyn std::error::Error>> + Send>;

pub struct Builder<R: tauri::Runtime = Wry> {
    inner: Mutex<GuiBuilder<R>>,
    setup_with_app_callback: Arc<Mutex<Vec<SetupHook<R>>>>,
}

impl<R> Builder<R>
where
    R: tauri::Runtime,
{
    pub async fn new() -> Result<Res<Self>, anyhow::Error> {
        let setup_with_app_callback: Arc<Mutex<Vec<SetupHook<R>>>> =
            Arc::new(Mutex::new(Vec::new()));
        let builder = {
            let setup_with_app_callback = setup_with_app_callback.clone();
            GuiBuilder::<R>::new().setup(move |app| {
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
        F: FnOnce(GuiBuilder<R>) -> Result<GuiBuilder<R>, anyhow::Error>,
    {
        self.replace(f(self.take())?);
        Ok(())
    }

    pub fn setup_with_app<F>(&self, f: F) -> &Self
    where
        F: FnOnce(&mut App<R>) -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
    {
        self.setup_with_app_callback
            .lock()
            .unwrap()
            .push(Box::new(f));
        self
    }

    pub fn take(&self) -> GuiBuilder<R> {
        self.replace(GuiBuilder::<R>::new())
    }

    fn replace(&self, builder: GuiBuilder<R>) -> GuiBuilder<R> {
        mem::replace(&mut self.inner.lock().unwrap(), builder)
    }
}
