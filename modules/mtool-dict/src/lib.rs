mod dict;
mod ui;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-dict");
    group.add_module(ui::module()).add_module(dict::Module);
    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-dict");
    group.add_module(ui::web_module()).add_module(dict::Module);
    group
}
