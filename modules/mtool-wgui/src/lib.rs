mod web;
pub use web::*;

cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        pub use web::module;
    } else {
        mod native;
        pub use native::*;
        pub use native::module;
    }
}
