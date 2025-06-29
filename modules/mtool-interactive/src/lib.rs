mod dioxus;
mod plugin;

// mod cmd;
// mod completion;
// mod ui;

// pub use completion::*;

use mapp::prelude::*;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-interactive");
    group.add_module(dioxus::Module);
    // group.add_module(cmd::Module);
    group
}

// pub fn web_module() -> LocalModuleGroup {
//     let mut group = LocalModuleGroup::new("mtool-interactive");
//     group.add_module(ui::web_module());

//     group.add_module(completion::Module);
//     group.add_module(cmd::Module);
//     group
// }
