use dioxus::prelude::*;
use mtool_dioxus::components::WindowView;

use crate::view::dock::DockView;

#[component]
pub fn MainView() -> Element {
    rsx! {
        if cfg!(target_os = "android") {
            document::Meta {
                http_equiv: "Content-Security-Policy",
                content: "upgrade-insecure-requests"
            }
        }
        document::Stylesheet {
            href: asset!("/assets/style.css")
        },
        WindowView {
            class: "border-none",
            tabindex: 0,
            autofocus: true,
            DockView {}
        }
    }
}
