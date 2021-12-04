mod hook;
mod key;

use std::{fmt::Debug, ptr::null_mut, sync::Mutex};

use anyhow::anyhow;
use anyhow::Context;
use once_cell::sync::OnceCell;
use thiserror::Error;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
        WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

use crate::{keydef::KeyCode, modifier_state::ModifierState, Event, KeyAction, KeyEvent};

use self::hook::GlobalHook;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Install hook failed")]
    InstallHook(WIN32_ERROR),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

static KEYBOARD_HOOK: OnceCell<GlobalHook> = OnceCell::new();
static MODIFIER_STATE: OnceCell<Mutex<ModifierState>> = OnceCell::new();
static EVENT_CALLBACK: OnceCell<Box<dyn Fn(Event) + Send + Sync>> = OnceCell::new();

extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ev = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let action = match wparam.0 as u32 {
        WM_KEYDOWN | WM_SYSKEYDOWN => KeyAction::Press,
        WM_KEYUP | WM_SYSKEYUP => KeyAction::Release,
        _ => panic!("Unknown action {}", wparam.0),
    };

    let keycode = KeyCode::from(ev.clone());
    let scancode = ev.scanCode;

    let modifiers = MODIFIER_STATE
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .update(&keycode, &action);

    let e = KeyEvent {
        scancode,
        keycode,
        modifiers,
        action,
    };

    EVENT_CALLBACK.get().unwrap()(Event::Key(e));

    unsafe { CallNextHookEx(KEYBOARD_HOOK.get().unwrap().handle(), code, wparam, lparam) }
}

pub fn run_loop<F>(cb: F) -> anyhow::Result<()>
where
    F: 'static + Fn(Event) + Send + Sync,
{
    if let Err(_) = EVENT_CALLBACK.set(Box::new(cb)) {
        return Err(anyhow!("Init event callback failed"));
    }

    if let Err(_) = MODIFIER_STATE.set(Mutex::new(ModifierState::new())) {
        return Err(anyhow!("Init modifier state failed"));
    }

    if let Err(_) = KEYBOARD_HOOK
        .set(GlobalHook::new(WH_KEYBOARD_LL, keyboard_hook).context("Register Keyboard event")?)
    {
        return Err(anyhow!("Init keyboard hook failed"));
    }

    unsafe {
        GetMessageW(null_mut(), HWND::default(), 0, 0);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        run_loop(|e| {
            println!("{:?}", e);
        })
        .unwrap();
    }
}
