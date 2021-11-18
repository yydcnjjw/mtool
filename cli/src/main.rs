use cloud_api::tencent;
use std::env;

// mod app;
// mod opts;
// mod command;
mod kbd;
mod keybind;

fn run_cmd(cmd: &String, args: &[String]) {
    println!("cmd {}, args {:?}", cmd, args);
}

#[tokio::main]
async fn main() {
    // let args = env::args().skip(1).collect::<Vec<String>>();
    // let (cmd, args) = args.split_first().unwrap();
    // run_cmd(cmd, args)

    // app::run().await.expect("Cli");
}
