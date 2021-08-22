use clap::Clap;

use crate::{
    command::{CommandRunner, SubCommand::*},
    config::Config,
    error::Result,
    opts::Opts,
};

pub struct App {
    pub opts: Opts,
    pub config: Config,
}

impl App {
    pub fn new() -> Result<App> {
        let opts = Opts::parse();
        
        let config = Config::load(&opts.config_path)?;

        Ok(App { opts, config })
    }

    pub async fn run(&self) -> Result<()> {
        Ok(match &self.opts.subcmd {
            Dict(dict) => dict.run().await?,
            Translate(translate) => translate.run().await?,
            Search(search) => search.run().await?,
            Ocr(ocr) => ocr.run(self).await?,
            Mdict(mdict) => mdict.run(self).await?,
        })
    }
}
