use anyhow::Context;
use windows::Win32::{
    Foundation::HINSTANCE,
    UI::WindowsAndMessaging::{
        SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, HOOKPROC, WINDOWS_HOOK_ID,
    },
};

#[derive(Debug)]
pub struct GlobalHook {
    inst: HHOOK,
}

impl GlobalHook {
    fn install(idhook: WINDOWS_HOOK_ID, hook: HOOKPROC) -> Result<HHOOK, anyhow::Error> {
        unsafe { SetWindowsHookExW(idhook, hook, HINSTANCE::default(), 0) }.context(format!("Failed to install hook: {:?}", idhook))
    }

    pub fn uninstall(&self) -> Result<(), anyhow::Error> {
        if unsafe { !UnhookWindowsHookEx(self.inst) }.as_bool() {
            anyhow::bail!("Failed to uninstall hook")
        } else {
            Ok(())
        }
    }

    pub fn handle(&self) -> HHOOK {
        self.inst
    }

    pub fn new(idhook: WINDOWS_HOOK_ID, hook: HOOKPROC) -> Result<Self, anyhow::Error> {
        Ok(Self {
            inst: GlobalHook::install(idhook, hook)?,
        })
    }
}
