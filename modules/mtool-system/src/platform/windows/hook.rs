use std::ffi::c_void;

use mapp::{
    anyhow,
    once_cell::sync::OnceCell,
    sync::RwLock,
    tracing::{info, warn},
};
use windows::Win32::{
    Foundation::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*,
};

pub struct Hook {
    handle: HHOOK,
}

pub struct HookData<Callback> {
    handle: u64, // TODO: Have alternative solutions without u64?
    callback: Callback,
}

type LowLevelKeybarodCallback = Box<dyn for<'a> Fn(LowLevelKeyboardEvent<'a>) + Send + Sync>;
type LowLevelKeyboardHookData = HookData<LowLevelKeybarodCallback>;

static LLKB_HOOK: OnceCell<RwLock<LowLevelKeyboardHookData>> = OnceCell::new();

impl Hook {
    pub fn global_low_level_keyboard_hook<Callback>(
        callback: Callback,
    ) -> Result<Hook, anyhow::Error>
    where
        Callback: for<'a> Fn(LowLevelKeyboardEvent<'a>) + Send + Sync + 'static,
    {
        info!("global low level keyboard hook");
        let handle = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_hook),
                Some(GetModuleHandleW(None)?.into()),
                0,
            )?
        };

        let hook_data = HookData {
            handle: handle.0 as u64,
            callback: Box::new(callback) as LowLevelKeybarodCallback,
        };

        if let Some(data) = LLKB_HOOK.get() {
            *data.write() = hook_data;
        } else {
            _ = LLKB_HOOK.set(RwLock::new(hook_data));
        }

        Ok(Hook { handle })
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        info!("global low level keyboard unhook");
        if let Err(e) = unsafe { UnhookWindowsHookEx(self.handle) } {
            warn!("{:?}", e);
        }
    }
}

pub struct LowLevelKeyboardEvent<'a> {
    pub lparam: &'a KBDLLHOOKSTRUCT,
    pub wparam: WPARAM,
}

extern "system" fn low_level_keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let handle = if let Some(hook_data) = LLKB_HOOK.get() {
        let hook = hook_data.read();

        if code as u32 == HC_ACTION {
            (hook.callback)(LowLevelKeyboardEvent {
                lparam: unsafe { (lparam.0 as *mut KBDLLHOOKSTRUCT).as_mut().unwrap() },
                wparam,
            })
        }

        Some(HHOOK(hook.handle as *mut c_void))
    } else {
        None
    };

    unsafe { CallNextHookEx(handle, code, wparam, lparam) }
}
