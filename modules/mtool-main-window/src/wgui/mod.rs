pub mod generic;
pub mod web;

mapp::cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        pub mod native;
    }
}

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub(crate) fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-main-window-wgui");
    group.add_module(native::Module);
    group
}

pub(crate) fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-main-window-wgui");
    group.add_module(web::Module);
    group
}
