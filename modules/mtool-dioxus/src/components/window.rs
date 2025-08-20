use dioxus::prelude::*;
use dioxus_primitives::toast::ToastProvider;

use crate::keybinding::Keybinding;

#[component]
pub fn WindowView(children: Element) -> Element {
    let keybinding = use_context_provider(provide_keybinding);

    let onkeydown = move |e| {
        keybinding.handle_web_key_down(e);
    };

    rsx! {
        document::Stylesheet {
            href: asset!("/assets/tailwind.css")
        },
        body {
            onkeydown,
            ToastProvider {
                children
            }
        }
    }
}

fn provide_keybinding() -> Keybinding {
    let keybinding = Keybinding::new();
    {
        let keybinding = keybinding.clone();
        spawn(async move { keybinding.run_loop().await });
    }
    keybinding
}
