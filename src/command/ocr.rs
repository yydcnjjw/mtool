use clap::Clap;
use ocr::config::Config;

use crate::{app::App, error};
use thiserror::Error;

use super::CommandRunner;
use async_trait::async_trait;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Ocr(#[from] ocr::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clap)]
pub struct OcrCmd {}

impl OcrCmd {
    async fn run_inner(&self, config: Config) -> Result<()> {
        Ok(ocr::app::App::new(config).run()?)
    }
}

#[async_trait]
impl CommandRunner for OcrCmd {
    async fn run(&self, app: &App) -> error::Result<()> {
        let config = app.config.get(&"ocr".to_string())?;
        Ok(self.run_inner(config).await?)
    }
}
