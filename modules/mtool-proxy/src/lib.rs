mod service;
mod ui;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-proxy");

    group.add_module(service::Module::default());

    group.add_module(ui::module());

    group
}
