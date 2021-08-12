use app::App;

mod app;
mod command;
mod config;
mod error;
mod opts;
mod util;

#[tokio::main]
async fn main() {
    let app = App::new().expect("App::new");
    app.run().await.expect("App::run");
}
