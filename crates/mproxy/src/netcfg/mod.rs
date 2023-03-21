cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod netlink;
        pub use netlink::*;
    }
}

mod interface;
mod routing;

pub use interface::*;
pub use routing::Routing;
