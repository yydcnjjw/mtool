#![feature(hash_set_entry)]

mod app;
mod core;
// mod module;
// mod opts;

use app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = App::run_loop().await {
        log::error!("{}", e);
    }
    Ok(())
}
