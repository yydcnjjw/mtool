use mapp::cfg_if::{self, cfg_if};

cfg_if! {
    if #[cfg(target_os = "android")] {
        mod android;

        pub use android::*;
    } else {
        mod dummy;

        pub use dummy::*;
    }
}
