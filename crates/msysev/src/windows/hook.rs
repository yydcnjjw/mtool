use windows::Win32::{
    Foundation::{GetLastError, HINSTANCE},
    UI::WindowsAndMessaging::{
        SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, HOOKPROC, WINDOWS_HOOK_ID,
    },
};

use crate::Error;

use super::Result;

#[derive(Debug)]
pub struct GlobalHook {
    inst: HHOOK,
}

impl GlobalHook {
    fn install(idhook: WINDOWS_HOOK_ID, hook: HOOKPROC) -> Result<HHOOK> {
        let hhk = unsafe { SetWindowsHookExW(idhook, hook, HINSTANCE::default(), 0) };
        Ok(hhk)
    }

    pub fn uninstall(&self) -> Result<()> {
        if unsafe { UnhookWindowsHookEx(self.inst) }.0 == 0 {
            Err(Error::UninstallHook(unsafe { GetLastError() }))
        } else {
            Ok(())
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
