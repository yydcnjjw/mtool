use std::{
    ffi::{c_void, CStr},
    os::raw::{c_char, c_int, c_ulong},
    ptr::{null, null_mut},
    sync::atomic::{AtomicPtr, Ordering},
};

use once_cell::sync::OnceCell;
use x11::{
    xlib::{self, _XDisplay},
    xrecord::{self, XRecordFreeContext},
};

use anyhow::Context;
use thiserror::Error;

use crate::{
    event::{Event, KeyAction, KeyEvent},
    keydef::{KeyCode, KeyModifier},
    linux::x11::key::KeySym,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    XLib(String),
    #[error("{0}")]
    XRecord(String),
    #[error("Init record failed")]
    Init,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

static RECORD: OnceCell<Record> = OnceCell::new();
static mut RECORD_ALL_CLIENTS: c_ulong = xrecord::XRecordAllClients;

pub struct Record {
    record_dpy: AtomicPtr<_XDisplay>,
    record_ctx: c_ulong,
    main_dpy: AtomicPtr<_XDisplay>,
    pub cb: Box<dyn Fn(Event) + Send + Sync>,
}

impl Drop for Record {
    fn drop(&mut self) {
        self.free_context();
        Self::close_display(self.record_dpy.load(Ordering::Relaxed)).unwrap();
        Self::close_display(self.main_dpy.load(Ordering::Relaxed)).unwrap();
    }
}

impl Record {
    fn open_display() -> Result<*mut _XDisplay> {
        let dpy = unsafe { xlib::XOpenDisplay(null()) };
        if dpy.is_null() {
            return Err(Error::XLib("Can't open display".into()));
        }
        Ok(dpy)
    }

    fn close_display(dpy: *mut _XDisplay) -> Result<()> {
        unsafe {
            if xlib::XCloseDisplay(dpy) as u8 == xlib::BadGC {
                return Err(Error::XLib("Can't close display".into()));
            }
            Ok(())
        }
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

    fn enable_context(&self) -> Result<()> {
        let result = unsafe {
            xrecord::XRecordEnableContext(
                self.record_dpy.load(Ordering::Relaxed),
                self.record_ctx,
                Some(record_cb),
                null_mut(),
            )
        };
        if result == 0 {
            return Err(Error::XRecord("Can't enable context".into()));
        }
        Ok(())
    }

    fn disable_context(&self) -> Result<()> {
        let result = unsafe {
            xrecord::XRecordDisableContext(self.record_dpy.load(Ordering::Relaxed), self.record_ctx)
        };
        if result == 0 {
            return Err(Error::XRecord("Can't disable context".into()));
        }
        Ok(())
    }

    fn free_context(&self) {
        unsafe {
            XRecordFreeContext(self.record_dpy.load(Ordering::Relaxed), self.record_ctx);
        }
    }

    fn new<F>(cb: F) -> Result<Self>
    where
        F: 'static + Fn(Event) + Send + Sync,
    {
        unsafe {
            let record_dpy = Self::open_display()?;
            let record_ctx = Self::create_context(record_dpy)?;

            xlib::XSync(record_dpy, xlib::True);

            let main_dpy = Self::open_display()?;

            Ok(Self {
                record_dpy: AtomicPtr::new(record_dpy),
                record_ctx,
                main_dpy: AtomicPtr::new(main_dpy),
                cb: Box::new(cb),
            })
        }
    }

    pub fn quit() -> Result<()> {
        RECORD.get().unwrap().disable_context()?;
        Ok(())
    }

    pub fn run_loop<F>(cb: F) -> Result<()>
    where
        F: 'static + Fn(Event) + Send + Sync,
    {
        if let Err(_) = RECORD.set(Record::new(cb)?) {
            return Err(Error::Init);
        }
        let record = RECORD.get().unwrap();
        record.enable_context()?;
        Ok(())
    }
}

unsafe extern "C" fn record_cb(_: *mut c_char, raw_data: *mut xrecord::XRecordInterceptData) {
    let record = RECORD.get().unwrap();

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
            let ks = xlib::XKeycodeToKeysym(record.main_dpy.load(Ordering::Relaxed), kc, 0);
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
