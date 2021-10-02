mod app;
mod service;

#[tokio::main]
async fn main() {
    app::App::new().await.expect("App").run().await;
}
