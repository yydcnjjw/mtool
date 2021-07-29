fn main() {
    cxx_build::bridge("src/lib.rs")
        .compile("screen");

    let dst = cmake::build("screenshot");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=dylib=screenshot");
}
