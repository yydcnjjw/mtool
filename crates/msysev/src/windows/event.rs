use std::{ptr, sync::Mutex};

use anyhow::Context as _;
use once_cell::sync::OnceCell;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, PeekMessageW, TranslateMessage, WaitMessage,
        KBDLLHOOKSTRUCT, MSG, PM_REMOVE, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
        WM_SYSKEYUP,
    },
};

use crate::{
    keydef::KeyCode, modifier_state::ModifierState, BoxedEventCallback, Event, KeyAction, KeyEvent,
};

use super::{hook::GlobalHook, Error};

struct Context {
    #[allow(dead_code)]
    hook_thread_id: u32,

    event_callback: BoxedEventCallback,

    keyboard_hook: GlobalHook,
    modifier_state: ModifierState,

    stop_guard: bool,
}

impl Context {
    fn new(cb: BoxedEventCallback) -> Result<Self, Error> {
        Ok(Self {
            hook_thread_id: unsafe { GetCurrentThreadId() },
            event_callback: cb,
            keyboard_hook: GlobalHook::new(WH_KEYBOARD_LL, Some(keyboard_hook))
                .context("Register Keyboard event")?,
            modifier_state: ModifierState::new(),
            stop_guard: false,
        })
    }
}

static CONTEXT: OnceCell<Mutex<Context>> = OnceCell::new();

pub fn run_loop(cb: BoxedEventCallback) -> Result<(), anyhow::Error> {
    CONTEXT
        .set(Mutex::new(Context::new(cb)?))
        .map_err(|_| anyhow::anyhow!("failed to init context"))?;

    let can_stop = || CONTEXT.get().unwrap().lock().unwrap().stop_guard;
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    log::debug!("system loop exited");

    Ok(())
}

pub fn quit() -> Result<(), anyhow::Error> {
    let mut guard = CONTEXT.get().unwrap().lock().unwrap();
    guard.keyboard_hook.uninstall()?;

    guard.stop_guard = true;

    Ok(())
}

extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ev = unsafe { *(lparam as *const KBDLLHOOKSTRUCT) };
    let action = match wparam as u32 {
        WM_KEYDOWN | WM_SYSKEYDOWN => KeyAction::Press,
        WM_KEYUP | WM_SYSKEYUP => KeyAction::Release,
        _ => panic!("Unknown action {}", wparam),
    };

    let keycode = KeyCode::from(ev.clone());
    let scancode = ev.scanCode;

    let mut guard = CONTEXT.get().unwrap().lock().unwrap();

    let modifiers = guard.modifier_state.update(&keycode, &action);

    let e = KeyEvent {
        scancode,
        keycode,
        modifiers,
        action,
    };

    if let Err(e) = (guard.event_callback)(Event::Key(e)) {
        log::error!("quit system event loop: {}", e);

        if let Err(e) = quit() {
            log::error!("Failed to quit: {}", e);
        }
    }

    unsafe { CallNextHookEx(guard.keyboard_hook.handle(), code, wparam, lparam) }
}
