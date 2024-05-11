use core::slice;
use std::io;

use tracing::warn;
use windows::Win32::{
    Foundation::*,
    System::{
        DataExchange::*,
        Memory::{GlobalLock, GlobalSize, GlobalUnlock},
        Ole::*,
    },
};

use crate::SelectionEvent;

struct ClipboardGuard;

impl ClipboardGuard {
    fn new() -> Result<Self, anyhow::Error> {
        unsafe {
            OpenClipboard(HWND::default())?;
        }
        Ok(Self)
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        if let Err(e) = unsafe { CloseClipboard() } {
            warn!("CloseClipboard failed: {:?}", e);
        }
    }
}

pub fn get_clipboard_data() -> Result<Option<SelectionEvent>, anyhow::Error> {
    let _guard = ClipboardGuard::new();

    let formats = vec![CF_UNICODETEXT.0 as u32];
    let format = unsafe { GetPriorityClipboardFormat(&formats) };

    if format == 0 {
        return Ok(None);
    } else if format == -1 {
        return Err(io::Error::last_os_error())?;
    }

    let data = unsafe { GetClipboardData(format as u32)? };

    Ok(match CLIPBOARD_FORMAT(format as u16) {
        CF_UNICODETEXT => {
            let glh = HGLOBAL(data.0 as *mut _);
            let text = unsafe { GlobalLock(glh) } as *const u16;
            if text.is_null() {
                return Err(io::Error::last_os_error())?;
            }

            let len = unsafe { GlobalSize(glh) } as usize / std::mem::size_of::<u16>();

            let text_slice = unsafe { slice::from_raw_parts(text, len - 1) };

            let text = String::from_utf16(text_slice)?;

            unsafe { GlobalUnlock(glh)? };

            Some(SelectionEvent {
                data: text.into(),
                mime_type: mime::TEXT_PLAIN_UTF_8,
            })
        }
        _ => unreachable!(),
    })
}
