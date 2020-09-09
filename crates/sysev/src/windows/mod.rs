mod hook;

use std::ptr::null_mut;

use anyhow::Context;
use once_cell::sync;
use thiserror::Error;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM},
    UI::WindowsAndMessaging::{CallNextHookEx, GetMessageW, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL},
};

use crate::Event;

use self::hook::GlobalHook;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Install hook failed")]
    InstallHook(WIN32_ERROR),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

static KEYBOARD_HOOK: sync::OnceCell<GlobalHook> = sync::OnceCell::new();

extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ev = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let action = wparam.0;

    println!("{:?}, {:?}", action, ev);

    unsafe { CallNextHookEx(KEYBOARD_HOOK.get().unwrap().handle(), code, wparam, lparam) }
}

pub fn run_loop<F>(cb: F) -> anyhow::Result<()>
where
    F: Fn(Event),
{
    KEYBOARD_HOOK
        .set(GlobalHook::new(WH_KEYBOARD_LL, keyboard_hook).context("Register Keyboard event")?)
        .unwrap();

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
        run_loop(|e| {}).unwrap();
    }
}
