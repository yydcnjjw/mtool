use mapp::{anyhow, prelude::*};

use crate::{create_system_info, event, keybinding};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(create_system_info);
        Ok(())
    }
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-system");

    group
        .add_module(Module)
        .add_module(event::Module)
        .add_module(keybinding::module());

    group
}
