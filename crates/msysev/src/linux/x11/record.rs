use std::{
    ffi::CStr,
    os::raw::{c_char, c_int, c_ulong},
    ptr::{null, null_mut},
    sync::atomic::{AtomicPtr, AtomicU64, Ordering},
};

use once_cell::sync::OnceCell;
use tracing::{error, warn};
use x11::{
    xlib::{self, _XDisplay},
    xrecord::{self, XRecordFreeContext, XRecordRange},
};

use anyhow::Context as _;

use crate::{
    event::{Event, KeyAction, KeyEvent},
    keydef::{KeyCode, KeyModifier},
    linux::x11::key::KeySym,
    BoxedEventCallback,
};

use super::Error;

static CONTEXT: OnceCell<Context> = OnceCell::new();
static mut RECORD_ALL_CLIENTS: c_ulong = xrecord::XRecordAllClients;

pub struct Context {
    record_ctx: AtomicU64,
    main_dpy: AtomicPtr<_XDisplay>,
    pub cb: BoxedEventCallback,
}

impl Context {
    unsafe fn open_display() -> Result<*mut _XDisplay, Error> {
        let dpy = xlib::XOpenDisplay(null());
        if dpy.is_null() {
            return Err(Error::XLib("Can't open display".into()));
        }
        Ok(dpy)
    }

    unsafe fn close_display(dpy: *mut _XDisplay) -> Result<(), Error> {
        if xlib::XCloseDisplay(dpy) as u8 == xlib::BadGC {
            return Err(Error::XLib("Can't close display".into()));
        }
        Ok(())
    }

    unsafe fn create_context(
        dpy: *mut _XDisplay,
        mut record_range: *mut XRecordRange,
    ) -> Result<c_ulong, Error> {
        let ext_name =
            CStr::from_bytes_with_nul(b"RECORD\0").context("Build CStr RECORD failed!")?;

        let extension = xlib::XInitExtension(dpy, ext_name.as_ptr());
        if extension.is_null() {
            return Err(Error::XLib("Can't init extension".into()));
        }

        let context =
            xrecord::XRecordCreateContext(dpy, 0, &mut RECORD_ALL_CLIENTS, 1, &mut record_range, 1);

        if context == 0 {
            return Err(Error::XRecord("Can't create context".into()));
        }

        Ok(context)
    }

    unsafe fn run_loop(&self) -> Result<(), Error> {
        let dpy = Self::open_display()?;

        let mut record_range = xrecord::XRecordAllocRange();
        (*record_range).device_events.first = xlib::KeyPress as u8;
        (*record_range).device_events.last = xlib::MotionNotify as u8;

        let record_ctx = Self::create_context(dpy, record_range)?;

        xlib::XSync(dpy, xlib::True);

        self.record_ctx.store(record_ctx, Ordering::Relaxed);

        let result = xrecord::XRecordEnableContext(dpy, record_ctx, Some(record_cb), null_mut());
        if result == 0 {
            return Err(Error::XRecord("Can't enable context".into()));
        }

        Self::close_display(dpy)?;

        Ok(())
    }

    unsafe fn quit(&self) -> Result<(), Error> {
        let dpy = Self::open_display()?;
        let ctx = self.record_ctx.load(Ordering::Relaxed);
        let result = xrecord::XRecordDisableContext(dpy, ctx);
        if result == 0 {
            return Err(Error::XRecord("Can't disable context".into()));
        }

        xlib::XFlush(dpy);
        xlib::XSync(dpy, xlib::False);
        XRecordFreeContext(dpy, ctx);
        Self::close_display(dpy)?;
        Ok(())
    }

    fn new(cb: BoxedEventCallback) -> Result<Self, Error> {
        unsafe {
            let main_dpy = Self::open_display()?;

            Ok(Self {
                record_ctx: AtomicU64::default(),
                main_dpy: AtomicPtr::new(main_dpy),
                cb,
            })
        }
    }
}

pub fn quit() -> Result<(), Error> {
    let ctx = CONTEXT.get().unwrap();
    unsafe {
        ctx.quit()?;
    }
    Ok(())
}

pub fn run_loop(cb: BoxedEventCallback) -> Result<(), Error> {
    if let Err(_) = CONTEXT.set(Context::new(cb)?) {
        return Err(Error::Init);
    }

    let ctx = CONTEXT.get().unwrap();
    unsafe {
        ctx.run_loop()?;
    }
    Ok(())
}

unsafe extern "C" fn record_cb(_: *mut c_char, raw_data: *mut xrecord::XRecordInterceptData) {
    let record = CONTEXT.get().unwrap();

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
        if let Err(e) = (record.cb)(e) {
            warn!("event callback error: {}", e);
            if let Err(e) = quit() {
                error!("Failed to quit system event loop: {}", e);
            }
        }
    }

    xrecord::XRecordFreeData(raw_data);
}
