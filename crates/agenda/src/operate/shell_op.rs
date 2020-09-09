use std::fmt;

use super::async_op::{self, AsyncOperate};
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use log::debug;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Executing command [{0}]: {1}")]
    CmdExec(std::process::ExitStatus, String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug)]
pub struct ShellOperate {
    script: String,
}

impl ShellOperate {
    pub fn new(script: String) -> Self {
        Self { script }
    }
}

impl fmt::Display for ShellOperate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ShellOperate: {}", self.script)
    }
}

#[async_trait]
impl AsyncOperate for ShellOperate {
    async fn run(&self) -> async_op::Result<()> {
        debug!("ShellOperate: {}", self.script);
        let output = Command::new("sh")
            .arg("-c")
            .arg(self.script.clone())
            .output()
            .await
            .with_context(|| format!("Executing script failed: {}", self.script))?;

        if !output.status.success() {
            return Err(anyhow!(Error::CmdExec(
                output.status,
                String::from_utf8(output.stderr).context("Convert output failed")?,
            ))
            .into());
        }

        Ok(())
    }
}
