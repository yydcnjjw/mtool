use std::{mem, sync::Mutex};

use mapp::provider::Res;

type GuiBuilder = tauri::Builder<tauri::Wry>;

pub struct Builder {
    inner: Mutex<GuiBuilder>,
}

impl Builder {
    pub async fn new() -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self {
            inner: Mutex::new(GuiBuilder::default()),
        }))
    }

    pub fn setup<F>(&self, f: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(GuiBuilder) -> Result<GuiBuilder, anyhow::Error>,
    {
        self.replace(f(self.take())?);
        Ok(())
    }

    pub fn take(&self) -> GuiBuilder {
        self.replace(GuiBuilder::default())
    }

    fn replace(&self, builder: GuiBuilder) -> GuiBuilder {
        mem::replace(&mut self.inner.lock().unwrap(), builder)
    }
}
