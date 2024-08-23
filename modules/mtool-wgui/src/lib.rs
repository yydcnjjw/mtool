mod web;
pub use web::*;
pub mod generic;

cfg_if::cfg_if! {
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
