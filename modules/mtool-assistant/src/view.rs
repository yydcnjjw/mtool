use dioxus::prelude::*;
use mtool_dioxus::components::WindowView;

pub fn assistant() -> Element {
    rsx! {
        WindowView {
            document::Stylesheet {
                href: asset!("/assets/style.css")
            },
            div {
                class: "text-white",
                height: "100vh",
            }
        }
    }
}
