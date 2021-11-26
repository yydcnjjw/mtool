#![feature(hash_set_entry)]

mod app;
mod core;
mod module;
// mod opts;

use app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    App::run().await?;
    Ok(())
}
