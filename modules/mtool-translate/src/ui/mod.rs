mod wgui;

#[cfg(not(target_family = "wasm"))]
mod cli;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-translate-ui");
    group.add_module(cli::Module);
    group.add_module(wgui::module());
    return group;
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-translate-ui");

    group.add_module(wgui::web_module());

    return group;
}
