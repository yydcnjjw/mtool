#![feature(hash_set_entry)]

mod app;
mod core;
mod module;

use app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    App::run_loop().await
}
