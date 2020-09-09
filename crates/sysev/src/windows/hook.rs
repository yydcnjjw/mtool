use windows::{
    core::Handle,
    Win32::{
        Foundation::{GetLastError, HINSTANCE},
        UI::WindowsAndMessaging::{SetWindowsHookExW, HHOOK, HOOKPROC, WINDOWS_HOOK_ID},
    },
};

use super::{Error, Result};

#[derive(Debug)]
pub struct GlobalHook {
    inst: HHOOK,
}

impl GlobalHook {
    fn install(idhook: WINDOWS_HOOK_ID, hook: HOOKPROC) -> Result<HHOOK> {
        let hhk = unsafe { SetWindowsHookExW(idhook, Some(hook), HINSTANCE::default(), 0) };

        if hhk.is_invalid() {
            Ok(hhk)
        } else {
            Err(Error::InstallHook(unsafe { GetLastError() }))
        }
    }

    pub fn handle(&self) -> HHOOK {
        self.inst
    }

    pub fn new(idhook: WINDOWS_HOOK_ID, hook: HOOKPROC) -> Result<Self> {
        Ok(Self {
            inst: GlobalHook::install(idhook, hook)?,
        })
    }
}
