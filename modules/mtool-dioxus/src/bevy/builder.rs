use bevy::prelude::*;
use mapp::{anyhow, sync::Mutex};

struct BevyAppBuilderInner {
    setup_hooks: Vec<Box<dyn (FnOnce(&mut App) -> Result<(), anyhow::Error>) + Send>>,
}

pub struct BevyAppBuilder {
    inner: Mutex<BevyAppBuilderInner>,
}

impl BevyAppBuilder {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(BevyAppBuilderInner {
                setup_hooks: Vec::new(),
            }),
        }
    }

    pub fn on_setup<F>(&self, hook: F)
    where
        F: (FnOnce(&mut App) -> Result<(), anyhow::Error>) + Send + 'static,
    {
        self.inner.lock().setup_hooks.push(Box::new(hook));
    }

    pub fn run_setup_hooks(&self, app: &mut App) -> Result<(), anyhow::Error> {
        for hook in self.inner.lock().setup_hooks.drain(..) {
            hook(app)?;
        }
        Ok(())
    }
}
