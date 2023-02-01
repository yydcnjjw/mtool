#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cli;
mod complete;
mod completion;
mod gui;
mod output;
mod tui;
mod utils;

use async_trait::async_trait;
pub use complete::*;
pub use completion::Completion;
pub use gui::GuiStage;
pub use output::OutputDevice;
pub use tauri::{AppHandle, GlobalShortcutManager, async_runtime};

use mapp::{AppContext, AppModule, ModuleGroup};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, _app: &mut AppContext) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::default();
    group
        .add_module(Module::default())
        .add_module(gui::Module::default())
        .add_module(cli::Module::default());
    group
}
