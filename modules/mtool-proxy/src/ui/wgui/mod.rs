mod app;
#[cfg(not(target_family = "wasm"))]
mod service;
mod stats;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-proxy-wgui");
    group.add_module(service::Module);
    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-proxy-wgui");
    group.add_module(app::Module);
    group
}
