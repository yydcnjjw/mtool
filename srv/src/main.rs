mod app;
mod service;

#[tokio::main]
async fn main() {
    app::App::new().expect("App").run().await;
}
