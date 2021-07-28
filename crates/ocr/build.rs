fn main() {
    let dst = cmake::build("screenshot");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=dylib=screenshot");

    cxx_build::bridge("src/lib.rs")
        .compile("screen")
}
