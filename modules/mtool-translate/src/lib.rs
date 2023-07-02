mod translator;
mod ui;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-translate");

    group.add_module(translator::Module);

    group.add_module(ui::module());

    return group;
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-translate");

    group.add_module(ui::web_module());

    return group;
}
