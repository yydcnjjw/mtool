use std::env;
use std::path::PathBuf;

fn get_cpp_link_stdlib(target: &str) -> Option<&'static str> {
    if target.contains("msvc") {
        None
    } else if target.contains("apple") || target.contains("freebsd") || target.contains("openbsd") {
        Some("c++")
    } else if target.contains("android") {
        Some("c++_shared")
    } else {
        Some("stdc++")
    }
}

fn main() {
    let target = env::var("TARGET").unwrap();
    // Link C++ standard library
    if let Some(cpp_stdlib) = get_cpp_link_stdlib(&target) {
        println!("cargo:rustc-link-lib=dylib={}", cpp_stdlib);
    }

    let build_type = "Release";

    let dst = cmake::Config::new("llama.cpp")
        .profile("Release")
        .define("LLAMA_OPENBLAS", "ON")
        .profile(build_type)
        // .define("LLAMA_OPENBLAS", "ON")
        .build_target("llama")
        .build();

    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build").display()
    );
    if cfg!(windows) {
        println!(
            "cargo:rustc-link-search=native={}",
            dst.join("build").join(build_type).display()
        );
    } else {
        println!(
            "cargo:rustc-link-search=native={}",
            dst.join("build").display()
        );
    }

    println!("cargo:rustc-link-lib=static=llama");

    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-Illama.cpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
