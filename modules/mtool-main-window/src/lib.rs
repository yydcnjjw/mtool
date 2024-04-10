pub mod wgui;

use mapp::prelude::*;

pub fn module() -> ModuleGroup {
    wgui::module()
}

pub fn web_module() -> LocalModuleGroup {
    wgui::web_module()
}
