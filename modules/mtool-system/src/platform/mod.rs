use mapp::cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "android")] {
        mod android;

        pub use android::*;
    } else if #[cfg(target_os = "windows")] {
        mod windows;

        pub use windows::*;
    } else {
        mod dummy;

        pub use dummy::*;
    }
}
