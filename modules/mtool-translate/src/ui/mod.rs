mod wgui;

#[cfg(not(target_family = "wasm"))]
mod cli;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-translate-ui");

    #[cfg(not(target_family = "wasm"))]
    group.add_module(cli::Module::default());

    group.add_module(wgui::module());

    return group;
}
