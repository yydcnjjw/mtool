mod completion;
mod output;
mod rand;
mod ui;

pub use completion::*;
pub use output::*;

use mapp::ModuleGroup;

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-interactive");
    group.add_module_group(ui::module());

    #[cfg(target_family = "wasm32")]
    group.add_module(completion::Module::default());
    group
}
