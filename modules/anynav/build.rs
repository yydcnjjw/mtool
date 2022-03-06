#[cfg(target_os = "windows")]
use embed_manifest::{embed_manifest, new_manifest};

fn main() {
    #[cfg(target_os = "windows")]
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest(new_manifest("anynav.manifest")).expect("unable to embed manifest file");
    }

    tauri_build::build();

    println!("cargo:rerun-if-changed=build.rs");    
}
