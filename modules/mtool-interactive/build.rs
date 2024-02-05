use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    if !target.contains("wasm") {
        const COMMANDS: &[&str] = &["complete", "complete_exit", "completion_meta"];
        tauri_plugin::Builder::new(COMMANDS).build();
    }
}
