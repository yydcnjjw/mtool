use std::{
    ffi::{c_void, CStr},
    os::raw::{c_char, c_int, c_ulong},
    ptr::null,
};

use x11::{
    xlib::{self, _XDisplay},
    xrecord,
};

use anyhow::Context;
use thiserror::Error;

use crate::{
    event::{Event, KeyAction, KeyEvent},
    keydef::{KeyCode, KeyModifier},
    linux::x11::key::KeySym,
    EventCallback,
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

pub struct Record {
    record_dpy: *mut _XDisplay,
    record_ctx: c_ulong,
    main_dpy: *mut _XDisplay,
    pub cb: EventCallback,
}

impl Record {
    unsafe fn open_display() -> Result<*mut _XDisplay> {
        let dpy = xlib::XOpenDisplay(null());
        if dpy.is_null() {
            return Err(Error::XLib("Can't open display".into()));
        }
        Ok(dpy)
    }

    unsafe fn create_context(dpy: *mut _XDisplay) -> Result<c_ulong> {
        let ext_name =
            CStr::from_bytes_with_nul(b"RECORD\0").context("Build CStr RECORD failed!")?;

        let extension = xlib::XInitExtension(dpy, ext_name.as_ptr());
        if extension.is_null() {
            return Err(Error::XLib("Can't init extension".into()));
        }

        let mut record_range = xrecord::XRecordAllocRange();
        (*record_range).device_events.first = xlib::KeyPress as u8;
        (*record_range).device_events.last = xlib::MotionNotify as u8;

        let context =
            xrecord::XRecordCreateContext(dpy, 0, &mut RECORD_ALL_CLIENTS, 1, &mut record_range, 1);

        if context == 0 {
            return Err(Error::XRecord("Can't create context".into()));
        }
        xlib::XFree(record_range as *mut c_void);

        Ok(context)
    }

    unsafe fn enable_context(record: *mut Record) -> Result<()> {
        let result = xrecord::XRecordEnableContext(
            (*record).record_dpy,
            (*record).record_ctx,
            Some(record_cb),
            record as *mut c_char,
        );
        if result == 0 {
            return Err(Error::XRecord("Can't enable context".into()));
        }
        Ok(())
    }

    fn new(cb: EventCallback) -> Result<Record> {
        unsafe {
            let record_dpy = Record::open_display()?;
            let record_ctx = Record::create_context(record_dpy)?;

            xlib::XSync(record_dpy, xlib::True);

            let main_dpy = Record::open_display()?;

            Ok(Record {
                record_dpy,
                record_ctx,
                main_dpy,
                cb,
            })
        }
    }

    pub fn run_loop(cb: EventCallback) -> Result<()> {
        unsafe {
            let mut r = Record::new(cb)?;
            Record::enable_context(&mut r)
        }
    }
}

unsafe extern "C" fn record_cb(record: *mut c_char, raw_data: *mut xrecord::XRecordInterceptData) {
    let record = &*(record as *mut Record);

    if (*raw_data).category != xrecord::XRecordFromServer {
        return;
    }

    let mut ev: Option<Event> = None;
    let xev = (*raw_data).data as *const xproto::_xEvent;

    let t = (*xev).u.u.as_ref().type_ as c_int;
    match t {
        xlib::KeyPress | xlib::KeyRelease => {
            let kc = (*xev).u.u.as_ref().detail;
            let modifiers = KeyModifier::from((*xev).u.keyButtonPointer.as_ref().state);
            let ks = xlib::XKeycodeToKeysym(record.main_dpy, kc, 0);
            let keycode = KeyCode::from(KeySym::new(ks));
            let scancode = (kc - 8) as u32;
            let action = if t == xlib::KeyPress {
                KeyAction::Press
            } else {
                KeyAction::Release
            };

            ev = Some(Event::Key(KeyEvent {
                scancode,
                keycode,
                modifiers,
                action,
            }));
        }
        _ => {}
    }

    if let Some(e) = ev {
        (record.cb)(e);
    }

    xrecord::XRecordFreeData(raw_data);
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn it_works() {}
}
