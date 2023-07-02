mod wgui;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-pdf-ui");
    group.add_module(wgui::module());
    group
}
