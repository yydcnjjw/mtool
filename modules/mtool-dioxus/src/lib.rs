pub mod bevy;
mod builder;
pub mod components;
mod custom_protocol;
pub mod hooks;
mod keybinding;
mod main_view;
mod module;
mod router;
mod window;

pub mod prelude {
    pub use crate::{
        builder::DioxusBuilder,
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
