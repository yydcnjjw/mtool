use std::{io, mem};

use tracing::warn;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::*, Graphics::Gdi::HBRUSH, System::SystemServices::IMAGE_DOS_HEADER,
        UI::WindowsAndMessaging::*,
    },
};

use crate::{windows::clipboard::get_clipboard_data, Event};

use super::event_loop::GLOBAL_EVENT_SENDER;

pub struct MessageOnlyWindow(HWND);

impl MessageOnlyWindow {
    pub fn new(class_name: PCSTR) -> Result<Self, anyhow::Error> {
        unsafe {
            let class = WNDCLASSA {
                style: WNDCLASS_STYLES(0),
                lpfnWndProc: Some(event_target_callback),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: get_instance_handle().into(),
                hIcon: HICON(0),
                hCursor: HCURSOR(0), // must be null in order for cursor state to work properly
                hbrBackground: HBRUSH(0),
                lpszMenuName: PCSTR::null(),
                lpszClassName: class_name.clone(),
            };

            RegisterClassA(&class);
        }

        let handle = unsafe {
            CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                class_name,
                PCSTR::null(),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                HWND_MESSAGE, // creates a message-only window
                HMENU::default(),
                get_instance_handle(),
                None,
            )
        };

        if handle == HWND::default() {
            return Err(io::Error::last_os_error())?;
        }

        Ok(Self(handle))
    }

    pub fn handle(&self) -> HWND {
        self.0.clone()
    }

    #[allow(unused)]
    pub fn destroy(mut self) -> Result<(), anyhow::Error> {
        unsafe { self.destroy_inner()? };
        mem::forget(self);
        Ok(())
    }

    unsafe fn destroy_inner(&mut self) -> Result<(), anyhow::Error> {
        Ok(unsafe { DestroyWindow(self.handle())? })
    }
}

impl Drop for MessageOnlyWindow {
    fn drop(&mut self) {
        let result = unsafe { self.destroy_inner() };
        debug_assert!(
            result.is_ok(),
            "MessageOnlyWindow::destroy() failed with {:?}",
            result.unwrap_err()
        );
    }
}

unsafe extern "system" fn event_target_callback(
    window: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLIPBOARDUPDATE => {
            match get_clipboard_data() {
                Ok(Some(data)) => {
                    if let Some(sender) = GLOBAL_EVENT_SENDER.get() {
                        let _ = sender.send(Event::Selection(data));
                    }
                }
                Err(e) => warn!("{:?}", e),
                _ => {}
            }

            LRESULT::default()
        }
        _ => unsafe { DefWindowProcW(window, msg, wparam, lparam) },
    }
}

pub fn get_instance_handle() -> HMODULE {
    // Gets the instance handle by taking the address of the
    // pseudo-variable created by the microsoft linker:
    // https://devblogs.microsoft.com/oldnewthing/20041025-00/?p=37483

    // This is preferred over GetModuleHandle(NULL) because it also works in DLLs:
    // https://stackoverflow.com/questions/21718027/getmodulehandlenull-vs-hinstance

    extern "C" {
        static __ImageBase: IMAGE_DOS_HEADER;
    }

    HMODULE(unsafe { &__ImageBase as *const _ as isize })
}
