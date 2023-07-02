#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(unsized_fn_params)]

// #![feature(async_closure)]
// #![feature(trait_alias)]

mod app;
mod label;
mod module;
pub mod provider;
mod schedule;
mod tracing;

pub mod prelude {

    pub use minject as inject;

    pub use crate::{
        app::*,
        label::*,
        module::{
            LocalModule as AppLocalModule, LocalModuleGroup, Module as AppModule, ModuleGroup,
        },
        provider::*,
        tracing::*,
        Error,
    };
}

pub use minject as inject;

pub use crate::{
    app::*,
    label::*,
    module::{LocalModule as AppLocalModule, LocalModuleGroup, Module as AppModule, ModuleGroup},
    schedule::*,
    tracing::*,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to initialize module {0}")]
    ModuleInit(&'static str, #[source] anyhow::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
