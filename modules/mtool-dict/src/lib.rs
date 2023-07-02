mod ui;
mod dict;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-dict");
    group.add_module(ui::module());
    group.add_module(dict::module());
    group
}


pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-dict");
    group.add_module(ui::web_module());
    group.add_module(dict::web_module());
    group
}
