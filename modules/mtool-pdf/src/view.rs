use dioxus::prelude::*;

pub fn pdf_viewer() -> Element {
    rsx! {
        div {
            class: "bg-transparent",
            height: "100vh",
            "Hello World!"
        }
    }
}
