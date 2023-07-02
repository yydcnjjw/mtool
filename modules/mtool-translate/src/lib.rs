mod translator;
mod ui;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-translate");

    group.add_module(translator::Module::default());

    group.add_module(ui::module());

    return group;
}
