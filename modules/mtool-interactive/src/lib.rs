mod cli;
mod complete;
mod completion;
mod output;
mod tui;
mod utils;
mod wgui;

pub use complete::*;
pub use completion::Completion;
pub use output::OutputDevice;
pub use tauri::{async_runtime, AppHandle, GlobalShortcutManager};

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("interactive_group");
    group
        .add_module(wgui::Module::default())
        .add_module(cli::Module::default());
    group
}
