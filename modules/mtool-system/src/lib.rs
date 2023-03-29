#![feature(arbitrary_self_types)]

pub mod event;
pub mod keybinding;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("system_group");

    group
        .add_module(event::Module::default())
        .add_module_group(keybinding::module());

    group
}
