use std::sync::{Arc, Mutex};

use once_cell::sync::OnceCell;
use tokio::sync::mpsc;
use tracing::debug;
use windows::{
    core::s,
    Win32::{
        Foundation::*,
        System::{
            DataExchange::{AddClipboardFormatListener, RemoveClipboardFormatListener},
            Threading::GetCurrentThreadId,
        },
        UI::WindowsAndMessaging::*,
    },
};

use crate::{windows::message_only_window::MessageOnlyWindow, Event};

// use super::hook::Hook;

pub struct PlatformEventLoop {
    exit_signal: ExitSignal,
}

#[derive(Clone)]
pub struct ExitSignal(Arc<Mutex<Option<u32>>>);

impl ExitSignal {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub fn exit(&self) -> Result<(), anyhow::Error> {
        let mut thread_id = self.0.lock().unwrap();
        if let Some(id) = thread_id.clone() {
            unsafe { PostThreadMessageW(id, WM_QUIT, WPARAM(0), LPARAM(0))? };
            *thread_id = None;
        }
        Ok(())
    }

    fn set_current_thread(&self) {
        *self.0.lock().unwrap() = Some(unsafe { GetCurrentThreadId() });
    }
}

pub static GLOBAL_EVENT_SENDER: OnceCell<mpsc::UnboundedSender<Event>> = OnceCell::new();

impl PlatformEventLoop {
    pub fn new(sender: mpsc::UnboundedSender<Event>) -> Result<Self, anyhow::Error> {
        let _ = GLOBAL_EVENT_SENDER.set(sender);

        Ok(Self {
            exit_signal: ExitSignal::new(),
        })
    }

    pub fn exit_signal(&self) -> ExitSignal {
        self.exit_signal.clone()
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        debug!(
            "system event loop run at {:?}",
            std::thread::current().name()
        );

        self.exit_signal.set_current_thread();

        // let _ll_keyboard_hook = Hook::global_low_level_keyboard_hook()?;

        let message_only_window = MessageOnlyWindow::new(s!("MTOOL_SYSTEM_EVENT_LOOP_WINDOW"))?;

        unsafe {
            AddClipboardFormatListener(message_only_window.handle())?;
        }

        let mut msg = MSG::default();
        unsafe {
            while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        unsafe {
            RemoveClipboardFormatListener(message_only_window.handle())?;
        }

        debug!("system event loop is exited");

        Ok(())
    }
}
