pub mod ecdict;
pub mod mdx;

use mapp::ModuleGroup;
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

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-dict-backend");
    group.add_module(mdx::Module);
    group.add_module(ecdict::Module);
    group
}
