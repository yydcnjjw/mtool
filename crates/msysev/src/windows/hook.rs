use once_cell::sync::OnceCell;
use tracing::{trace, warn};
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

use crate::{
    keyboard::*,
    windows::{event_loop::GLOBAL_EVENT_SENDER, keyboard::*},
    Event, KeyEvent,
};

pub struct Hook(pub HHOOK);

impl Hook {
    pub fn global_low_level_keyboard_hook() -> Result<Hook, anyhow::Error> {
        Ok(Self(unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_hook),
                HMODULE::default(),
                0,
            )?
        }))
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        if let Err(e) = unsafe { UnhookWindowsHookEx(self.0) } {
            warn!("{:?}", e);
        }
    }
}

static mut MODIFIER_STATE: OnceCell<ModifierState> = OnceCell::with_value(ModifierState::NONE);

extern "system" fn low_level_keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ev = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let state = match wparam.0 as u32 {
        WM_KEYDOWN | WM_SYSKEYDOWN => KeyState::Press,
        WM_KEYUP | WM_SYSKEYUP => KeyState::Release,
        _ => panic!("Unknown state {:?}", wparam),
    };

    let key = scancode_to_physicalkey(ev.scanCode);

    let modifiers = if let Some(modifiers) = unsafe { MODIFIER_STATE.get_mut() } {
        update_modifier_state(modifiers, &key, &state);
        *modifiers
    } else {
        ModifierState::NONE
    };

    let e = KeyEvent {
        key,
        modifiers,
        state,
    };

    trace!("low level keyboard event: {:?}", e);

    if let Some(sender) = GLOBAL_EVENT_SENDER.get() {
        let _ = sender.send(Event::Key(e));
    }

    unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) }
}
