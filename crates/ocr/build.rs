fn main() {
    cxx_build::bridge("src/app.rs").compile("app");

    let dst = cmake::build("screenshot");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=screenshot");

    pkg_config::probe_library("Qt5Quick").unwrap();

    println!("cargo:rustc-link-lib=dylib=Qt5Quick");

    qt_ritual_build::add_resources(concat!(env!("CARGO_MANIFEST_DIR"), "/screenshot/qml/screenshot.qrc"));
}
