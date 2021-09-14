#![feature(macro_attributes_in_derive_output)]

mod app;
mod opts;
mod command;

#[tokio::main]
async fn main() {
    app::run().await.expect("Cli");
}
