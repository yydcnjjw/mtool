mod app;
mod opts;
mod command;

#[tokio::main]
async fn main() {
    app::run().await.expect("Cli");
}
