use mapp::cfg_if::cfg_if;

mod netease;
mod player;
mod remote;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::*;
    } else if #[cfg(target_os = "android")] {
        mod android;
        pub use android::*;
    }
}

pub use netease::*;
pub use player::*;
pub use remote::*;
