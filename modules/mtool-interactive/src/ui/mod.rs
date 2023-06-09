#[cfg(not(target_family = "wasm"))]
mod cli;

pub mod wgui;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-interactive-ui");

    #[cfg(not(target_family = "wasm"))]
    group.add_module(cli::Module::default());

    group.add_module_group(wgui::module());

    group
}
