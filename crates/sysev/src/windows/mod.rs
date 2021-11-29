use std::ptr::null_mut;

use thiserror::Error;
use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, SetWindowsHookExW, HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL,
    },
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

static mut HHK: HHOOK = HHOOK(0);

extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ev = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let action = wparam.0;

    println!("{:?}, {:?}", action, ev);

    unsafe { CallNextHookEx(HHK, code, wparam, lparam) }
}

fn run() -> Result<()> {
    unsafe {
        HHK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), HINSTANCE::default(), 0);
        GetMessageW(null_mut(), HWND::default(), 0, 0);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        run().unwrap();
    }
}
