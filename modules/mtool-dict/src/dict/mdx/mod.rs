mapp::cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod dict;
        pub use dict::*;
    }
}

mod view;
pub use view::*;
