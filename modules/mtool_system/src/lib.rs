pub mod event;
mod keybinding;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::default();

    group
        .add_module(event::Module::default())
        .add_module(keybinding::Module::default());

    group
}
