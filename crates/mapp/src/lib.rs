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

pub use minject as inject;

pub use app::*;
pub use label::*;
pub use module::{Module as AppModule, ModuleGroup};
// pub use provider::*;
pub use schedule::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to initialize module {0}")]
    ModuleInit(&'static str, #[source] anyhow::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
