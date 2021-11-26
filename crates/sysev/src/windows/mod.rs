use std::{thread, time::Duration};

use thiserror::Error;
use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, PeekMessageW, TranslateMessage, HHOOK, MSG, PM_REMOVE,
        WH_KEYBOARD,
    },
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

static mut HHK: HHOOK = HHOOK(0);

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    CallNextHookEx(HHK, code, wparam, lparam)
}

fn run() -> Result<()> {
    unsafe {
        HHK = windows::Win32::UI::WindowsAndMessaging::SetWindowsHookExW(
            WH_KEYBOARD,
            Some(keyboard_hook),
            HINSTANCE::default(),
            0,
        );

        let mut msg = MSG::default();
        loop {
            if !PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            thread::sleep(Duration::from_millis(200));
        }
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
