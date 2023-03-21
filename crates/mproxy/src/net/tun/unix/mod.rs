pub mod async_io;
pub mod block_io;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux_tun;
        pub use linux_tun::*;
    }
}
