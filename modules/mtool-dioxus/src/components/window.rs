use dioxus::prelude::*;

#[component]
pub fn WindowView(children: Element) -> Element {
    rsx! {
        document::Stylesheet {
            href: asset!("/assets/tailwind.css")
        },
        { children }
    }
}
