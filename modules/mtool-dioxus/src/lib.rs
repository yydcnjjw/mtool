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
mod action;

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

pub use dioxus_desktop as desktop;
pub use dioxus_free_icons as free_icons;
pub use dioxus_primitives as primitives;
