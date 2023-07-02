mod web;

pub use web::*;

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod native;
        pub use native::*;
    }
}
