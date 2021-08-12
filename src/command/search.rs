use clap::Clap;

use crate::error::Result;

#[derive(Clap)]
pub struct SearchCmd {}

impl SearchCmd {
    pub async fn run(&self) -> Result<()>{
        // search::run();
        Ok(())
    }
}
