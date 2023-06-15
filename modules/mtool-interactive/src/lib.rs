mod output;
mod completion;
mod rand;  
mod ui;

pub use completion::*;
pub use output::*;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-interactive");
    group.add_module_group(ui::module());
    group
}
