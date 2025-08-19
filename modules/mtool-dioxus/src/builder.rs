use std::{collections::HashMap, mem, ops::DerefMut, sync::Arc};

use dioxus::prelude::*;
use mapp::{anyhow, sync::Mutex};

pub type GlobalHotkeys =
    Arc<HashMap<String, Arc<dyn Fn() -> Result<(), anyhow::Error> + Send + Sync>>>;

struct DioxusBuilderInner {
    dioxus_hooks: Vec<
        Box<
            dyn FnOnce(
                    dioxus_desktop::Config,
                    LaunchBuilder,
                ) -> (dioxus_desktop::Config, LaunchBuilder)
                + Send,
        >,
    >,
    launch_builder_hooks: Vec<Box<dyn FnOnce(LaunchBuilder) -> LaunchBuilder + Send>>,
    config_builder_hooks:
        Vec<Box<dyn FnOnce(dioxus_desktop::Config) -> dioxus_desktop::Config + Send>>,
    global_hotkeys: HashMap<String, Arc<dyn Fn() -> Result<(), anyhow::Error> + Send + Sync>>,
    launcher: LaunchFn,
}

impl DioxusBuilderInner {
    fn new() -> Self {
        Self {
            dioxus_hooks: Vec::new(),
            launch_builder_hooks: Vec::new(),
            config_builder_hooks: Vec::new(),
            global_hotkeys: HashMap::new(),
            launcher: dioxus_desktop::launch::launch,
        }
    }

    fn with_launcher(&mut self, launch_fn: LaunchFn) -> &mut Self {
        self.launcher = launch_fn;
        self
    }

    fn with_launch_builder<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(LaunchBuilder) -> LaunchBuilder + Send + 'static,
    {
        self.launch_builder_hooks.push(Box::new(f));
        self
    }

    fn with_config_builder<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(dioxus_desktop::Config) -> dioxus_desktop::Config + Send + 'static,
    {
        self.config_builder_hooks.push(Box::new(f));
        self
    }

    fn with_dioxus<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(dioxus_desktop::Config, LaunchBuilder) -> (dioxus_desktop::Config, LaunchBuilder)
            + Send
            + 'static,
    {
        self.dioxus_hooks.push(Box::new(f));
        self
    }

    fn add_global_hotkey<T, Callback>(&mut self, hotkey: T, cb: Callback) -> &mut Self
    where
        T: ToString,
        Callback: Fn() -> Result<(), anyhow::Error> + Send + Sync + 'static,
    {
        self.global_hotkeys
            .entry(hotkey.to_string())
            .or_insert(Arc::new(cb));
        self
    }

    fn launch(self, app: fn() -> Element) {
        let mut config = dioxus_desktop::Config::new();

        for hook in self.config_builder_hooks {
            config = hook(config)
        }

        let mut launch_builder = LaunchBuilder::custom(self.launcher);
        for hook in self.launch_builder_hooks {
            launch_builder = hook(launch_builder)
        }

        for hook in self.dioxus_hooks {
            (config, launch_builder) = hook(config, launch_builder)
        }

        launch_builder = launch_builder.with_context(Arc::new(self.global_hotkeys));

        launch_builder.with_cfg(config).launch(app);
    }
}

pub struct DioxusBuilder {
    inner: Mutex<DioxusBuilderInner>,
}

impl DioxusBuilder {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(DioxusBuilderInner::new()),
        }
    }

    pub fn with_launcher(&self, launch_fn: LaunchFn) -> &Self {
        self.inner.lock().with_launcher(launch_fn);
        self
    }

    pub fn with_dioxus<F>(&self, f: F) -> &Self
    where
        F: FnOnce(dioxus_desktop::Config, LaunchBuilder) -> (dioxus_desktop::Config, LaunchBuilder)
            + Send
            + 'static,
    {
        self.inner.lock().with_dioxus(f);
        self
    }

    pub fn with_launch_builder<F>(&self, f: F) -> &Self
    where
        F: FnOnce(LaunchBuilder) -> LaunchBuilder + Send + 'static,
    {
        self.inner.lock().with_launch_builder(f);
        self
    }

    pub fn with_config_builder<F>(&self, f: F) -> &Self
    where
        F: FnOnce(dioxus_desktop::Config) -> dioxus_desktop::Config + Send + 'static,
    {
        self.inner.lock().with_config_builder(f);
        self
    }

    pub fn add_global_hotkey<T, Callback>(&self, hotkey: T, cb: Callback) -> &Self
    where
        T: ToString,
        Callback: Fn() -> Result<(), anyhow::Error> + Send + Sync + 'static,
    {
        self.inner.lock().add_global_hotkey(hotkey, cb);
        self
    }

    pub(crate) fn launch(&self, app: fn() -> Element) {
        let mut builder = self.inner.lock();
        let builder = mem::replace(builder.deref_mut(), DioxusBuilderInner::new());
        builder.launch(app);
    }
}
