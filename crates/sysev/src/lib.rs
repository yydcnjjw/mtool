#![feature(type_alias_impl_trait)]

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

mod event;
pub mod keydef;

pub use event::*;
