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

fn add_compile_define<'a>(compiler: &'a mut cc::Build, target: &str) -> &'a mut cc::Build {
    if target.contains("msvc") {
        compiler.define("_CRT_SECURE_NO_WARNINGS", None)
    } else {
        compiler
    }
}

fn build_ggml(target: &str) {
    let mut compiler = cc::Build::new();
    add_compile_define(&mut compiler, target);
    compiler
        .warnings(false)
        .include("llama.cpp")
        .file("llama.cpp/ggml.c")
        .compile("ggml");
    println!("cargo:rustc-link-lib=static=ggml");
}

fn build_llama(target: &str) {
    let mut compiler = cc::Build::new();
    add_compile_define(&mut compiler, target);
    compiler
        .cpp(true)
        .warnings(false)
        .include("llama.cpp")
        .file("llama.cpp/llama.cpp")
        .cpp_link_stdlib(get_cpp_link_stdlib(target))
        .compile("llama");
    println!("cargo:rustc-link-lib=static=llama");
}

fn main() {
    let target = env::var("TARGET").unwrap();

    build_ggml(&target);
    build_llama(&target);

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
