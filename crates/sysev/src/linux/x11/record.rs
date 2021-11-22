use std::{
    any::Any,
    ffi::{c_void, CStr},
    os::raw::{c_char, c_int, c_uint, c_ulong},
    ptr::{null, null_mut},
};

use x11::{
    keysym::XK_M,
    xlib::{self, XkbGetNames, _XDisplay},
    xrecord,
};

use anyhow::Context;
use thiserror::Error;

use crate::{
    event::{self, KeyAction, KeyCode},
    linux::x11::keysym::KeySym,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    XLib(String),
    #[error("{0}")]
    XRecord(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

static mut RECORD_ALL_CLIENTS: c_ulong = xrecord::XRecordAllClients;

fn run() -> Result<()> {
    unsafe {
        let display = xlib::XOpenDisplay(null());
        if display.is_null() {
            return Err(Error::XLib("Can't open display".into()));
        }

        let ext_name =
            CStr::from_bytes_with_nul(b"RECORD\0").context("Build CStr RECORD failed!")?;

        let extension = xlib::XInitExtension(display, ext_name.as_ptr());
        if extension.is_null() {
            return Err(Error::XLib("Can't init extension".into()));
        }

        let mut record_range = xrecord::XRecordAllocRange();
        (*record_range).device_events.first = xlib::KeyPress as u8;
        (*record_range).device_events.last = xlib::MotionNotify as u8;

        let context = xrecord::XRecordCreateContext(
            display,
            0,
            &mut RECORD_ALL_CLIENTS,
            1,
            &mut record_range,
            1,
        );

        if context == 0 {
            return Err(Error::XRecord("Can't create context".into()));
        }

        xlib::XFree(record_range as *mut c_void);

        xlib::XSync(display, xlib::True);

        let display2 = xlib::XOpenDisplay(null());
        if display2.is_null() {
            return Err(Error::XLib("Can't open display".into()));
        }

        let result = xrecord::XRecordEnableContext(
            display,
            context,
            Some(record_callback),
            display2 as *mut c_char,
        );
        if result == 0 {
            return Err(Error::XRecord("Can't enable context".into()));
        }
    }
    Ok(())
}

unsafe extern "C" fn record_callback(
    display: *mut c_char,
    raw_data: *mut xrecord::XRecordInterceptData,
) {
    if (*raw_data).category != xrecord::XRecordFromServer {
        return;
    }

    let ev = (*raw_data).data as *const xproto::_xEvent;

    match (*ev).u.u.as_ref().type_ as c_int {
        xlib::KeyPress => {
            let keycode = (*ev).u.u.as_ref().detail;
            let state = (*ev).u.keyButtonPointer.as_ref().state;

            let keysym = xlib::XKeycodeToKeysym(
                display as *mut _XDisplay,
                keycode,
                0
                // if (state as u32 & xlib::ShiftMask) != 0 {
                //     1
                // } else {
                //     0
                // },
            );

            let s = KeyCode::from(KeySym::new(keysym));
            println!("{}, {}, {:?}, {:?}", keycode, state, keysym, s);
        }
        _ => {}
    }

    xrecord::XRecordFreeData(raw_data);
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn it_works() {
        run().unwrap();
    }
}
