#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

mod event;
mod modifier_state;
pub mod keydef;

pub use event::*;