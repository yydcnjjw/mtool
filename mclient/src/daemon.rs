#[cfg(not(target_os = "windows"))]
use crate::path;
#[cfg(not(target_os = "windows"))]
use anyhow::Context;
#[cfg(not(target_os = "windows"))]
use daemonize::Daemonize;

#[cfg(target_os = "windows")]
pub fn daemon() -> anyhow::Result<()> {
    log::warn!("daemon not supported at windows");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn daemon() -> anyhow::Result<()> {
    let daemonize = Daemonize::new()
        .pid_file(
            path::config_dir()
                .map(|p| p.join("m.pid"))
                .context("Failed to get pid file path")?,
        )
        .working_directory(path::config_dir().context("Failed to get m config path")?);
    daemonize.start().context("Failed to start daemon")?;
    Ok(())
}
