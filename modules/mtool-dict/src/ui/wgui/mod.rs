mod app;

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod service;
        mod native;
    }
}

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-dict-wgui");

    #[cfg(target_family = "wasm")]
    group.add_module(app::Module);

    cfg_if::cfg_if! {
        if #[cfg(not(target_family = "wasm"))] {
            group.add_module(service::Module);
            group.add_module(native::Module);
        }
    }
    group
}
