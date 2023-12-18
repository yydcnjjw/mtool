mod web;

pub use web::*;

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod native;
        mod service;
    }
}

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-pdf-wgui");

    group.add_module(native::Module);
    group.add_module(service::Module);

    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-pdf-wgui");

    group.add_module(web::Module);

    group
}
