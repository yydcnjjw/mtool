use app::App;

mod app;
mod error;
mod command;
mod config;
mod opts;
mod util;

#[tokio::main]
async fn main() {
    App::new().unwrap().run().await.unwrap();
}
