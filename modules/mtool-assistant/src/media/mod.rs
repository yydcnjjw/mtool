mod module;
mod netease;
mod player;
mod remote;

pub use module::*;
pub use netease::*;
pub use player::*;
pub use remote::*;

mapp::cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::*;
    } else if #[cfg(target_os = "android")] {
        mod android;
        pub use android::*;
    }
}
