use mapp::ModuleGroup;

mod wgui;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-dict-ui");
    group.add_module_group(wgui::module());
    group
}
