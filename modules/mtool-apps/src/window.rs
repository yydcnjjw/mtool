use image::DynamicImage;
use mapp::{anyhow, prelude::*};
use mtool_cmdpal::{CommandResult, InvokeCommand};

use crate::platforms;

#[derive(Clone)]
pub struct WindowInfo {
    pub title: String,
    #[allow(unused)]
    pub position: WindowPosition,
    pub process: ProcessInfo,

    pub(crate) inner: platforms::WindowInfo,
}

#[derive(Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub description: String,
    #[allow(unused)]
    pub pid: usize,
}

pub trait WindowInfoExt {
    fn load_icon(&self) -> Result<DynamicImage, anyhow::Error>;
    fn active(&self) -> Result<(), anyhow::Error>;
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct ActiveWindowCommand {
    win_info: WindowInfo,
}

impl ActiveWindowCommand {
    pub fn new(win_info: WindowInfo) -> Self {
        Self { win_info }
    }
}

#[async_trait]
impl InvokeCommand for ActiveWindowCommand {
    async fn invoke(&self) -> Result<CommandResult, anyhow::Error> {
        self.win_info.active()?;
        Ok(CommandResult::Dismiss)
    }
}
