mod app;
#[cfg(not(target_family = "wasm"))]
mod service;
mod stats;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-proxy-wgui");

    #[cfg(target_family = "wasm")]
    group.add_module(app::Module::default());

    #[cfg(not(target_family = "wasm"))]
    group.add_module(service::Module::default());

    group
}
