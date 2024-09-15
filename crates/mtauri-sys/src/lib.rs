#![feature(trait_alias)]

mod error;
mod event;
mod tauri;
mod window;
// pub mod os;

pub mod prelude {
    pub use crate::{
        event::{listen, Event},
        tauri::{invoke, invoke_raw},
        window::{UnlistenFn, Window},
    };
}

pub use error::IntoAnyhowError;
