#![feature(arbitrary_self_types)]

mod ui;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-pdf");

    group.add_module(ui::module());
    
    group
}
