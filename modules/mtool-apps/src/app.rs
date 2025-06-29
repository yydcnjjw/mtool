use image::DynamicImage;
use mapp::{anyhow, itertools::Itertools, prelude::*, tracing::warn};
use mtool_cmdpal::{Command, CommandItem, CommandResult, InvokeCommand};
use pinyin::{to_pinyin_vec, Pinyin, ToPinyin};
use std::sync::Arc;

use crate::{base64_image, platforms};

#[derive(Clone)]
pub struct AppInfo {
    pub name: String,
    pub description: Option<String>,

    pub(crate) inner: platforms::AppInfo,
}

pub trait AppInfoExt {
    fn load_icon(&self) -> Result<DynamicImage, anyhow::Error>;
    fn launch(&self) -> Result<(), anyhow::Error>;
}

impl From<&AppInfo> for CommandItem {
    fn from(app_info: &AppInfo) -> Self {
        let name = &app_info.name;
        Self {
            title: name.clone(),
            sub_title: app_info.description.clone().unwrap_or_default(),
            icon: app_info
                .load_icon()
                .and_then(|icon| base64_image(icon))
                .inspect_err(|e| warn!("load app icon: {:#}", e))
                .ok(),
            hints: {
                app_info
                    .description
                    .iter()
                    .chain([name])
                    .flat_map(|v| {
                        [
                            v.to_owned(),
                            v.as_str()
                                .to_pinyin()
                                .filter_map(|v| v.map(Pinyin::plain))
                                .join(" "),
                        ]
                    })
                    .filter(|v| !v.trim().is_empty())
                    .collect_vec()
            },
            source: String::new(),
            command: Arc::new(Command::new(
                "apps.launch.app",
                LaunchAppCommand::new(app_info.clone()),
            )),
            more_commands: Vec::new(),
        }
    }
}

pub struct LaunchAppCommand {
    app_info: AppInfo,
}

impl LaunchAppCommand {
    fn new(app_info: AppInfo) -> Self {
        Self { app_info }
    }
}

#[async_trait]
impl InvokeCommand for LaunchAppCommand {
    async fn invoke(&self) -> Result<CommandResult, anyhow::Error> {
        self.app_info.launch()?;
        Ok(CommandResult::Dismiss)
    }
}
