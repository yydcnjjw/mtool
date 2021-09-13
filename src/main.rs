#![feature(macro_attributes_in_derive_output)]

use app::{App, Result};

mod app;
mod command;
mod config;
mod error;
mod opts;
mod util;

async fn run() -> Result<()> {
    App::new()?.run().await
}

#[tokio::main]
async fn main() {
    run().await.expect("App");
}
