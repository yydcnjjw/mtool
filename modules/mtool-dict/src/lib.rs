mod ui;
mod dict;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-dict");
    group.add_module_group(ui::module());
    group.add_module_group(dict::module());
    group
}
