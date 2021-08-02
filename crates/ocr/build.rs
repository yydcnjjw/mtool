fn main() {
    cxx_build::bridge("src/lib.rs")
        .include("/usr/include/qt/QtCore")
        .include("/usr/include/qt")
        .include("/usr/include/qt/QtQml")
        .include("/usr/include/qt/QtGui")
        .include("/usr/include/qt/QtQuick")
        .compile("screen");
 
    let dst = cmake::build("screenshot");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=dylib=screenshot");

    // println!("cargo:rustc-link-search=native={}", dst.display());
    // println!("cargo:rustc-link-lib=static=screenshot-static");

    qt_ritual_build::add_resources(concat!(env!("CARGO_MANIFEST_DIR"), "/screenshot/qml.qrc"));
}
