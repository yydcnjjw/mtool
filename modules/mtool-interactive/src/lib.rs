mod cli;
mod complete;
mod completion;
mod gui;
mod output;
mod tui;
mod utils;

pub use complete::*;
pub use completion::Completion;
pub use output::OutputDevice;
pub use tauri::{AppHandle, GlobalShortcutManager, async_runtime};

use mapp::{AppContext, AppModule, ModuleGroup};
use async_trait::async_trait;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, _app: &mut AppContext) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("interactive_group");
    group
        .add_module(Module::default())
        .add_module(gui::Module::default())
        .add_module(cli::Module::default());
    group
}
