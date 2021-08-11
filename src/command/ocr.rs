use clap::Clap;

use crate::{app::App, error::Result};

#[derive(Clap)]
pub struct Ocr {}

impl Ocr {
    pub async fn run(&self, app: &App) -> Result<()> {
        ocr::run(app.config.get(&"ocr".to_string())?);
        Ok(())
    }
}
