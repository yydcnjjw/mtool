mod migration;
pub mod entity;

use mapp::prelude::*;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-pdf-storage");
    group.add_module(migration::Module);
    group
}
