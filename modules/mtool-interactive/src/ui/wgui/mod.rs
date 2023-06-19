mod web;
pub use web::*;

cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        pub use web::Module;
    } else {
        mod service;
        pub use service::*;
        pub use service::Module;
    }
}

mod model;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-interactive-wgui");

    group.add_module(Module::default());

    group
}
