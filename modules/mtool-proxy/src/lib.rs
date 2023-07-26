mod service;
mod ui;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-proxy");

    group.add_module(service::Module);

    group.add_module(ui::module());

    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-proxy");

    group.add_module(service::Module);

    group.add_module(ui::web_module());

    group
}
