use anyhow::Context;
use daemonize::Daemonize;

use crate::path;

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
