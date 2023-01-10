mod hook;
mod key;

pub mod event;

use thiserror::Error;
use windows::Win32::Foundation::WIN32_ERROR;

#[derive(Debug, Error)]
pub enum Error {
    #[allow(dead_code)]
    #[error("Install hook failed")]
    InstallHook(WIN32_ERROR),
    #[error("Uninstall hook failed")]
    UninstallHook(WIN32_ERROR),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
