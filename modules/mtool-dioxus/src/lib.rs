#[cfg(feature = "bevy")]
pub mod bevy;
mod builder;
pub mod components;
mod context;
mod custom_protocol;
pub mod hooks;
mod keybinding;
mod launch;
mod main_view;
mod message;
mod module;
mod router;
mod window;
mod platform;

pub mod prelude {
    pub use crate::{
        builder::DioxusBuilder,
        context::*,
        hooks::*,
        keybinding::*,
        module::DioxusStage,
        router::{app_route, RouteParams, Router},
        window::*,
    };
}

pub use module::module;

pub use winit;
pub use dioxus_free_icons as free_icons;
pub use dioxus_primitives as primitives;


#[cfg(feature = "native")]
pub use dioxus_native as platform;

#[cfg(feature = "webview")]
pub use dioxus_desktop as platform;
