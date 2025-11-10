use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    if target.contains("windows") {
        println!("cargo::rustc-link-arg-bins=/ENTRY:mainCRTStartup")
    }
}
