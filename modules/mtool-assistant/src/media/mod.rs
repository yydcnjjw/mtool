mod media_item;
mod media_metadata;
mod media_source;
mod module;
mod player;
mod subtitle_track;
mod timed_metadata_source;

pub use media_item::*;
pub use media_metadata::*;
pub use media_source::*;
pub use module::*;
pub use player::*;
pub use subtitle_track::*;
pub use timed_metadata_source::*;

mapp::cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::*;
    } else if #[cfg(target_os = "android")] {
        mod android;
        pub use android::*;
    }
}
