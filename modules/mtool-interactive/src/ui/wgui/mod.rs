cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod service;
        pub use service::Completion;
    }

}

mod model;
mod web;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-interactive-wgui");
    group.add_module(service::Module);
    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-interactive-wgui");

    group.add_module(web::Module);

    group
}
