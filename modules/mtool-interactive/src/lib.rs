mod completion;
mod rand;
mod ui;

pub use completion::*;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-interactive");
    group.add_module(ui::module());
    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-interactive");
    group.add_module(ui::web_module());

    group.add_module(completion::Module);
    group
}
