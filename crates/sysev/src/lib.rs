#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

mod event;
pub mod keydef;

pub use linux::*;
pub use event::*;
