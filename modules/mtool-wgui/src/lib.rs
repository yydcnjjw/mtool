mod web;
pub use web::*;
pub mod generic;

mapp::cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod native;
        pub use native::*;
    }
}

pub mod prelude {
    #[cfg(not(target_family = "wasm"))]
    pub use crate::native::*;

    pub use crate::web::*;
}

pub use gloo_events;
pub use gloo_utils;
pub use grass;
pub use js_sys;
pub use mtauri_sys;
pub use send_wrapper;
// pub use yew;
// pub use yew_autoprops;
// pub use yew_router;

mapp::cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        pub use tauri_plugin;
    }
}
