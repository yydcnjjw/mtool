#![feature(hash_set_entry)]
mod app;
// mod opts;
mod command;
mod kbd;
mod keybind;

use app::App;
use std::env;

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
