#![allow(incomplete_features)]
#![feature(iterator_try_collect, trait_alias)]

mod config;
pub mod stats;

pub use config::protos;

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod io;
        mod net;
        pub mod proxy;
        pub mod router;

        #[cfg(feature = "telemetry")]
        pub mod metrics;
        
        mod app;
        pub use app::*;
        
        pub use config::AppConfig;
    }
}
