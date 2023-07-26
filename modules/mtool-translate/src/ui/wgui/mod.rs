mod app;

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod native;
        mod service;
    }
}

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-translate-wgui");

    group.add_module(native::Module::default());
    group.add_module(service::Module::default());

    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-translate-wgui");

    group.add_module(app::Module);

    group
}
