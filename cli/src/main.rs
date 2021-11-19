mod app;
// mod opts;
mod command;
// mod kbd;
// mod keybind;

use anyhow::Context;
use app::App;
use command::Command;
use mytool_core::config;
use std::{env, path::PathBuf};

async fn run() -> anyhow::Result<()> {
    let mut app = App::new().await?;

    command::add_command(&mut app)?;

    let args = env::args().skip(1).collect::<Vec<String>>();
    let (cmd, args) = args.split_first().unwrap();

    app.cmder.exec(cmd, args).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await?;
    Ok(())
}
