use std::any::type_name;

use anyhow::Context;
use async_trait::async_trait;

use crate::app::AppContext;

#[async_trait]
pub trait Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error>;

    fn name(&self) -> &'static str {
        type_name::<Self>()
    }
}

#[derive(Default)]
pub struct ModuleGroup {
    modules: Vec<Box<dyn Module + Send + Sync>>,
}

impl ModuleGroup {
    pub fn add_module_group(&mut self, module: ModuleGroup) -> &mut Self {
        self.modules.extend(module.modules);
        self
    }

    pub fn add_module<Mod>(&mut self, module: Mod) -> &mut Self
    where
        Mod: Module + Send + Sync + 'static,
    {
        self.modules.push(Box::new(module));
        self
    }
}

#[async_trait]
impl Module for ModuleGroup {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        for module in &self.modules {
            let name = module.name();
            module
                .init(ctx)
                .await
                .context(format!("Failed to init {} module", name))?;
        }
        Ok(())
    }
}
