mod ffi {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(js_namespace = ["__TAURI_OS_PLUGIN_INTERNALS__"])]
    extern "C" {
        #[wasm_bindgen(thread_local, js_name = platform)]
        pub static PLATFORM: String;
    }
}

pub enum Platform {
    Linux,
    Macro,
    Ios,
    Freebsd,
    Dragonfly,
    Netbsd,
    Openbsd,
    Solaris,
    Android,
    Windows,
}

pub fn platform() -> Platform {
    ffi::PLATFORM.with(|v| match v.as_str() {
        "linux" => Platform::Linux,
        "macos" => Platform::Macro,
        "ios" => Platform::Ios,
        "freebsd" => Platform::Freebsd,
        "dragonfly" => Platform::Dragonfly,
        "netbsd" => Platform::Netbsd,
        "openbsd" => Platform::Openbsd,
        "solaris" => Platform::Solaris,
        "android" => Platform::Android,
        "windows" => Platform::Windows,
        _ => unreachable!(),
    })
}
