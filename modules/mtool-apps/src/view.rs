use dioxus::prelude::*;
use mtool_cmdpal::{components::CommandList, CommandSource};

use crate::AppsSource;

#[component]
pub fn Apps() -> Element {
    let app_source: AppsSource = use_context();

    let mut items = use_signal(Vec::new);
    use_hook(|| {
        spawn(async move {
            let mut rx = app_source.subscribe();
            items.set(rx.borrow().clone());
            while let Ok(_) = rx.changed().await {
                items.set(rx.borrow().clone());
            }
        })
    });

    rsx! {
        SuspenseBoundary {
            fallback: |_| rsx! {
                div {
                    width: "100%",
                    height: "100%",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",
                    "Loading..."
                }
            },
            CommandList {
                items
            },
        }
    }
}
