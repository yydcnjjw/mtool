#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

mod event;

#[cfg(feature = "event-loop")]
mod event_loop;

mod keyboard;

pub mod prelude {
    pub use crate::{
        event::{Event, KeyEvent, SelectionEvent},
        keyboard::{KeyCode, KeyState, ModifierState, NativeKeyCode, PhysicalKey},
    };
    pub use mime::*;
}

#[cfg(feature = "event-loop")]
pub use event_loop::{ControlFlow, EventLoop, ExitSignal};
