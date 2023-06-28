pub mod wgui;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-interactive-ui");

    group.add_module_group(wgui::module());

    group
}
