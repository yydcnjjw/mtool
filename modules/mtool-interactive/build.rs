#![feature(exit_status_error)]
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=webapp/src");
    println!("cargo:rerun-if-changed=webapp/index.html");
    Command::new("trunk")
        // NOTE: To avoid pollution of env
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .args([
            "build",
            "--dist",
            "../mtool-gui/webapp/interactive",
            "--public-url",
            "/interactive",
            "--release",
            "webapp/index.html",
        ])
        .status()
        .unwrap()
        .exit_ok()
        .unwrap();
}
