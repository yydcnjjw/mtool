mod app;

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod service;
        mod native;
    }
}

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-dict-wgui");
    group.add_module(service::Module);
    group.add_module(native::Module);
    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-dict-wgui");
    group.add_module(app::Module);
    group
}
