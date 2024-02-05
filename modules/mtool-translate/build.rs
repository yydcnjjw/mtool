use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    if !target.contains("wasm") {
        const COMMANDS: &[&str] = &["text_translate"];
        tauri_plugin::Builder::new(COMMANDS).build();
    }
}
