use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use anyhow::Context as _;
use once_cell::sync::OnceCell;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, TranslateMessage, KBDLLHOOKSTRUCT, MSG,
        WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

use crate::{
    keydef::KeyCode, modifier_state::ModifierState, BoxedEventCallback, Event, KeyAction, KeyEvent,
};

use super::hook::GlobalHook;

struct Context {
    #[allow(dead_code)]
    hook_thread_id: u32,

    event_callback: BoxedEventCallback,

    keyboard_hook: GlobalHook,
    modifier_state: ModifierState,

    can_stop: bool,
}

impl Context {
    fn new(cb: BoxedEventCallback) -> Result<Self, anyhow::Error> {
        Ok(Self {
            hook_thread_id: unsafe { GetCurrentThreadId() },
            event_callback: cb,
            keyboard_hook: GlobalHook::new(WH_KEYBOARD_LL, Some(keyboard_hook))
                .context("Register Keyboard event")?,
            modifier_state: ModifierState::new(),
            can_stop: false,
        })
    }

    fn get() -> RwLockReadGuard<'static, Context> {
        CONTEXT.get().unwrap().read().unwrap()
    }

    fn get_mut() -> RwLockWriteGuard<'static, Context> {
        CONTEXT.get().unwrap().write().unwrap()
    }
}

static CONTEXT: OnceCell<RwLock<Context>> = OnceCell::new();

pub fn run_loop(cb: BoxedEventCallback) -> Result<(), anyhow::Error> {
    CONTEXT
        .set(RwLock::new(Context::new(cb)?))
        .map_err(|_| anyhow::anyhow!("failed to init context"))?;

    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);

            if Context::get().can_stop {
                break;
            }
        }
    }

    log::debug!("system loop exited");

    Ok(())
}

pub fn quit() -> Result<(), anyhow::Error> {
    let mut ctx = Context::get_mut();
    ctx.keyboard_hook.uninstall()?;

    ctx.can_stop = true;

    Ok(())
}

extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ev = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let action = match wparam.0 as u32 {
        WM_KEYDOWN | WM_SYSKEYDOWN => KeyAction::Press,
        WM_KEYUP | WM_SYSKEYUP => KeyAction::Release,
        _ => panic!("Unknown action {:?}", wparam),
    };

    let keycode = KeyCode::from(ev.clone());
    let scancode = ev.scanCode;

    let modifiers = { Context::get_mut().modifier_state.update(&keycode, &action) };

    let e = KeyEvent {
        scancode,
        keycode,
        modifiers,
        action,
    };

    log::debug!("receive event: {:?}", e);

    {
        if let Err(e) = (Context::get().event_callback)(Event::Key(e)) {
            log::error!("quit system event loop: {}", e);

            if let Err(e) = quit() {
                log::error!("Failed to quit: {}", e);
            }
        }
    }

    unsafe { CallNextHookEx(Context::get().keyboard_hook.handle(), code, wparam, lparam) }
}