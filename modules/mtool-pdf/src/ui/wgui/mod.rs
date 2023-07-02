mod app;

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod native;
        mod service;
    }
}

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-translate-wgui");

    #[cfg(target_family = "wasm")]
    group.add_module(app::Module);

    #[cfg(not(target_family = "wasm"))]
    {
        group.add_module(native::Module);
        group.add_module(service::Module);
    }

    group
}
