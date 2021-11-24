#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

pub use linux::*;

pub mod event;
pub mod event_bus;

pub mod keydef;
