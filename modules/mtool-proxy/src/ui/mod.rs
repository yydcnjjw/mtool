mod wgui;

use mapp::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    wgui::module()
}

pub fn web_module() -> LocalModuleGroup {
    wgui::web_module()
}
