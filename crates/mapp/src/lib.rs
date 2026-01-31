#![feature(trait_alias)]

// #![feature(unsize)]
// #![feature(coerce_unsized)]
// #![feature(unsized_fn_params)]

// mod app;
// mod error;
// mod label;
// mod module;
// mod platform;
// mod trace;
// mod utils;

pub mod context;
pub mod coroutine;
pub mod hash;
pub mod inject;
pub mod intern;
pub mod label;
pub mod lazy;
pub mod schedule;

// Required to make proc macros work in bevy itself.
extern crate self as mapp;

pub mod prelude {
    // pub use crate::{
    //     app::{AppContext, LocalAppContext},
    //     error::Error as AppError,
    //     label::Label,
    //     module::{
    //         LocalModule as AppLocalModule, LocalModuleGroup, Module as AppModule, ModuleGroup,
    //     },
    //     // provider::{Injector, LocalInjector, Res, Take, TakeOpt},
    //     schedule::{ExitSignal, ScheduleGraph},
    //     trace::Tracing,
    //     utils::rand_string,
    // };

    pub use crate::{context::*, label::*, schedule::*};
    // pub use crate::coroutine::*;
    // pub use crate::inject::*;
}

pub use self::schedule::{ScheduleLabel, TaskSet};

// pub use app::{AppBuilder, LocalAppBuilder};
// pub use schedule::Label;
// #[allow(unused)]
// pub use platform::*;
// pub use schedule::{CreateLocalOnceTaskDescriptor, CreateOnceTaskDescriptor};

// pub use anyhow;
// pub use async_recursion;
// pub use cfg_if;
// pub use dashmap;
// pub use dpi;
// pub use futures;
// pub use itertools;
// pub use keyboard_types;
// pub use nom;
// pub use notify_rust;
// pub use once_cell;
// pub use parking_lot as sync;
// pub use pin_project_lite;
// pub use rand;
// pub use regex;
// pub use reqwest;
// pub use scopeguard;
// pub use send_wrapper;
// pub use serde;
// pub use serde_error;
// pub use serde_json;
// pub use serde_with;
// pub use tokio;
// pub use tokio_stream;
// pub use tokio_util;
// pub use toml;
// pub use tracing;
// pub use tracing_appender;
// pub use tracing_subscriber;
// pub use url;
// pub use base64;

// #[cfg(feature = "ai")]
// pub use rig;

// cfg_if::cfg_if! {
//     if #[cfg(target_os = "android")] {
//         pub use android_activity;
//     }
// }
