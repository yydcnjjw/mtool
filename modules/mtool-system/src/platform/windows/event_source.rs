use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    thread,
};

use mapp::{
    anyhow,
    keyboard_types::{Code, KeyState, Modifiers},
    prelude::*,
    tokio::sync::broadcast,
    tracing::warn,
};
use mtool_dioxus::prelude::*;
use windows::Win32::{
    Foundation::*, System::Threading::GetCurrentThreadId, UI::WindowsAndMessaging::*,
};

use crate::{platform::windows::keyboard::scancode_to_physicalkey, Keyboard, SystemEvent};

use super::hook::{Hook, LowLevelKeyboardEvent};

pub struct SystemEventSource {
    source: broadcast::Sender<SystemEvent>,
    loop_thread_id: Arc<AtomicU32>,
}

impl Drop for SystemEventSource {
    fn drop(&mut self) {
        let id = self.loop_thread_id.load(Ordering::Relaxed);
        if id != 0 {
            unsafe {
                if let Err(e) = PostThreadMessageW(id, WM_QUIT, WPARAM(0), LPARAM(0)) {
                    warn!("{e}")
                }
            };
        }
    }
}

impl SystemEventSource {
    pub async fn new(_context: Res<DioxusContext>) -> Result<SystemEventSource, anyhow::Error> {
        let (source, _) = broadcast::channel(64);
        let loop_thread_id = Arc::new(AtomicU32::new(0));

        {
            let sender = source.clone();
            let loop_thread_id = loop_thread_id.clone();
            thread::spawn(move || {
                loop_thread_id.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
                if let Err(e) = Self::system_event_loop(sender) {
                    warn!("{e:?}");
                }
            });
        }

        Ok(SystemEventSource {
            source,
            loop_thread_id,
        })
    }

    fn system_event_loop(sender: broadcast::Sender<SystemEvent>) -> Result<(), anyhow::Error> {
        let _llkbh = Hook::global_low_level_keyboard_hook(move |ev| {
            Self::handle_llkb_event(&sender, ev);
        })?;

        let mut msg = MSG::default();
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        Ok(())
    }

    fn handle_llkb_event(
        sender: &broadcast::Sender<SystemEvent>,
        LowLevelKeyboardEvent { lparam, wparam }: LowLevelKeyboardEvent,
    ) {
        if let Some(code) = scancode_to_physicalkey(lparam.scanCode) {
            let state = match wparam.0 as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => KeyState::Down,
                WM_KEYUP | WM_SYSKEYUP => KeyState::Up,
                _ => return,
            };

            let modifiers = update_modifier_state(&code, &state);

            if let Err(e) = sender.send(SystemEvent::Keyboard(Keyboard {
                state,
                code,
                modifiers,
            })) {
                warn!("{e:?}");
            }
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.source.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<SystemEvent> {
        self.source.clone()
    }
}

pub fn update_modifier_state(code: &Code, state: &KeyState) -> Modifiers {
    static mut MODIFIERS: Modifiers = Modifiers::empty();

    let modifer = match code {
        Code::ShiftLeft | Code::ShiftRight => Modifiers::SHIFT,
        Code::CapsLock => Modifiers::CAPS_LOCK,
        Code::ControlLeft | Code::ControlRight => Modifiers::CONTROL,
        Code::AltLeft | Code::AltRight => Modifiers::ALT,
        Code::NumLock => Modifiers::NUM_LOCK,
        Code::Super => Modifiers::SUPER,
        _ => Modifiers::empty(),
    };

    unsafe {
        match state {
            KeyState::Down => MODIFIERS |= modifer,
            KeyState::Up => MODIFIERS -= modifer,
        };
        MODIFIERS
    }
}
