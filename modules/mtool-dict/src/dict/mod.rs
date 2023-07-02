pub mod ecdict;
pub mod mdx;

use mapp::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Backend {
    Mdx,
    ECDict,
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Backend::Mdx => write!(f, "mdx"),
            Backend::ECDict => write!(f, "ecdict"),
        }
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-dict-backend");
    group.add_module(mdx::Module);
    group.add_module(ecdict::Module);
    group
}

pub fn web_module() -> LocalModuleGroup {
    let mut group = LocalModuleGroup::new("mtool-dict-backend");
    group.add_module(mdx::Module);
    group.add_module(ecdict::Module);
    group
}
