#![feature(arbitrary_self_types, result_option_inspect, iterator_try_collect, slice_group_by)]

#[cfg(not(target_family = "wasm"))]
mod pdf;

mod ui;

mod config;
pub use config::*;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-pdf");

    group.add_module(ui::module());
    group.add_module(pdf::Module);

    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-pdf");

    group.add_module(ui::web_module());

    group
}
