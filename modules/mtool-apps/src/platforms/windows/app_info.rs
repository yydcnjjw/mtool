use image::DynamicImage;
use mapp::anyhow::{self, anyhow, Context};
use parselnk::Lnk;
use std::path::{Path, PathBuf};
use windows_icons::get_icon_by_path;

use crate::AppInfoExt;

#[derive(Clone)]
pub struct AppInfo {
    path: PathBuf,
    lnk: Lnk,
}

impl AppInfo {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<crate::AppInfo, anyhow::Error> {
        let path = path.as_ref();
        let lnk = Lnk::try_from(path)?;
        Ok(crate::AppInfo {
            name: path
                .file_stem()
                .map(|v| format!("{}", v.display()))
                .unwrap_or(format!("{}", path.display())),
            description: lnk.description(),
            inner: Self {
                path: path.to_path_buf(),
                lnk,
            },
        })
    }
}

impl AppInfoExt for crate::AppInfo {
    fn load_icon(&self) -> Result<DynamicImage, anyhow::Error> {
        let path = &self.inner.path;
        let path_str = path
            .to_str()
            .ok_or(anyhow!("parse path to str {}", path.display()))?;

        let icon = get_icon_by_path(path_str).map_err(|e| anyhow!("{}", e))?;

        Ok(DynamicImage::from(icon))
    }

    fn launch(&self) -> Result<(), anyhow::Error> {
        let path = &self.inner.path;
        open::that_detached(path).context(format!("{}", path.display()))
    }
}
