use emacs::Transfer;
use mapp::prelude::*;

pub(crate) struct EmacsContext {
    pub injector: Injector,
}

impl Transfer for EmacsContext {}
