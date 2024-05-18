mod generic;
mod web;

#[cfg(not(target_family = "wasm"))]
mod native;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-proxy-wgui");
    group.add_module(native::Module);
    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-proxy-wgui");
    group.add_module(web::Module);
    group
}
