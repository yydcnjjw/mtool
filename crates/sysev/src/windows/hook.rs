use windows::Win32::{
    Foundation::HINSTANCE,
    UI::WindowsAndMessaging::{SetWindowsHookExW, HHOOK, HOOKPROC, WINDOWS_HOOK_ID},
};

use super::Result;

#[derive(Debug)]
pub struct GlobalHook {
    inst: HHOOK,
}

impl GlobalHook {
    fn install(idhook: WINDOWS_HOOK_ID, hook: HOOKPROC) -> Result<HHOOK> {
        let hhk = unsafe { SetWindowsHookExW(idhook, Some(hook), HINSTANCE::default(), 0) };
        Ok(hhk)
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
