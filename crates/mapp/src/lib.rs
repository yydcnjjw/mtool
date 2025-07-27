// #![feature(unsize)]
// #![feature(coerce_unsized)]
// #![feature(unsized_fn_params)]

mod app;
mod error;
mod label;
mod module;
pub mod provider;
mod schedule;
mod trace;
mod utils;
mod platform;

pub mod prelude {
    pub use crate::{
        app::{AppContext, LocalAppContext},
        error::Error as AppError,
        label::Label,
        module::{
            LocalModule as AppLocalModule, LocalModuleGroup, Module as AppModule, ModuleGroup,
        },
        provider::{Injector, LocalInjector, Res, Take, TakeOpt},
        schedule::ScheduleGraph,
        trace::Tracing,
        utils::rand_string,
    };

    pub use async_trait::*;
    pub use minject::*;
}

pub use app::{AppBuilder, LocalAppBuilder};
pub use platform::*;
pub use label::Label;
pub use schedule::{CreateLocalOnceTaskDescriptor, CreateOnceTaskDescriptor};

pub use anyhow;
pub use async_recursion;
pub use cfg_if;
pub use dashmap;
pub use dpi;
pub use futures;
pub use itertools;
pub use keyboard_types;
pub use nom;
pub use notify_rust;
pub use once_cell;
pub use parking_lot as sync;
pub use pin_project_lite;
pub use rand;
pub use regex;
pub use reqwest;
pub use scopeguard;
pub use send_wrapper;
pub use serde;
pub use serde_error;
pub use serde_json;
pub use serde_with;
pub use tokio;
pub use tokio_stream;
pub use tokio_util;
pub use toml;
pub use tracing;
pub use tracing_appender;
pub use tracing_subscriber;

#[cfg(target_os = "android")]
pub use android_activity;
