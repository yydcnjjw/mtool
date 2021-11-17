mod app;
mod opts;
mod command;
mod kbd;
mod keybind;

#[tokio::main]
async fn main() {
    app::run().await.expect("Cli");
}
