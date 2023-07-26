use std::any::type_name;

use anyhow::Context;
use async_trait::async_trait;
use tracing::{instrument, trace};

use crate::{app::AppContext, LocalAppContext};

#[async_trait]
pub trait Module: Send + Sync {
    fn early_init(&self, _ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error>;

    fn name(&self) -> &'static str {
        type_name::<Self>()
    }
}

#[derive(Default)]
pub struct ModuleGroup {
    name: Option<&'static str>,
    modules: Vec<Box<dyn Module>>,
}

impl ModuleGroup {
    pub fn new(name: &'static str) -> Self {
        Self {
            name: Some(name),
            modules: Vec::default(),
        }
    }

    pub fn add_module<Mod>(&mut self, module: Mod) -> &mut Self
    where
        Mod: Module + 'static,
    {
        self.modules.push(Box::new(module));
        self
    }
}

#[async_trait]
impl Module for ModuleGroup {
    #[instrument(name = "module_group", skip_all, fields(name = self.name()))]
    fn early_init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        trace!(target: "module", "group early_init");

        for module in &self.modules {
            let name = module.name();

            trace!(target: "module", "early_init {}", name);

            module
                .early_init(ctx)
                .context(format!("Failed to early init {} module", name))?;
        }

        Ok(())
    }

    #[instrument(name = "module_group", skip_all, fields(name = self.name()))]
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        trace!(target: "module", "group init");

        for module in &self.modules {
            let name = module.name();

            trace!(target: "module", "init {}", name);

            module
                .init(ctx)
                .await
                .context(format!("Failed to init {} module", name))?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        self.name.unwrap_or(type_name::<Self>())
    }
}

#[async_trait(?Send)]
pub trait LocalModule {
    fn local_early_init(&self, _ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error>;

    fn name(&self) -> &'static str {
        type_name::<Self>()
    }
}

#[derive(Default)]
pub struct LocalModuleGroup {
    name: Option<&'static str>,
    modules: Vec<Box<dyn LocalModule>>,
}

impl LocalModuleGroup {
    pub fn new(name: &'static str) -> Self {
        Self {
            name: Some(name),
            modules: Vec::default(),
        }
    }

    pub fn add_module<Mod>(&mut self, module: Mod) -> &mut Self
    where
        Mod: LocalModule + 'static,
    {
        self.modules.push(Box::new(module));
        self
    }
}

#[async_trait(?Send)]
impl LocalModule for LocalModuleGroup {
    #[instrument(name = "local_module_group", skip_all, fields(name = self.name()))]
    fn local_early_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        trace!(target: "local_module", "group early_init");

        for module in &self.modules {
            let name = module.name();

            trace!(target: "module", "early_init {}", name);

            module
                .local_early_init(ctx)
                .context(format!("Failed to early init {} module", name))?;
        }

        Ok(())
    }

    #[instrument(name = "local_module_group", skip_all, fields(name = self.name()))]
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        trace!(target: "local_module", "group init");

        for module in &self.modules {
            let name = module.name();

            trace!(target: "module", "init {}", name);

            module
                .local_init(ctx)
                .await
                .context(format!("Failed to init {} module", name))?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        self.name.unwrap_or(type_name::<Self>())
    }
}
