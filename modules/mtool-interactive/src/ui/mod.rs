pub mod wgui;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-interactive-ui");

    group.add_module(wgui::module());

    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-interactive-ui");

    group.add_module(wgui::web_module());

    group
}
