use clap::Clap;

use crate::error::Result;

#[derive(Clap)]
pub struct Search {}

impl Search {
    pub async fn run(&self) -> Result<()>{
        // search::run();
        Ok(())
    }
}
