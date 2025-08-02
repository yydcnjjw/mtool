use mapp::{
    anyhow,
    prelude::*,
    sync::{Mutex, MutexGuard},
};

pub struct SystemInfo {
    inner: Mutex<sysinfo::System>,
}

impl SystemInfo {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(sysinfo::System::new()),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, sysinfo::System> {
        self.inner.lock()
    }
}

pub(crate) async fn create_system_info() -> Result<Res<SystemInfo>, anyhow::Error> {
    if sysinfo::IS_SUPPORTED_SYSTEM {
        Ok(Res::new(SystemInfo::new()))
    } else {
        anyhow::bail!("OS is unsupported")
    }
}
