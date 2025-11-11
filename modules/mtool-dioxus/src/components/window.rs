use dioxus::prelude::*;
use dioxus_desktop::{
    use_wry_event_handler,
    winit::event::{DeviceEvent::Key, Event::DeviceEvent},
};
use dioxus_primitives::toast::ToastProvider;

use crate::keybinding::Keybinding;

#[component]
pub fn WindowView(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let keybinding = use_context_provider(provide_keybinding);

    use_wry_event_handler(move |ev, _event_loop| match ev {
        DeviceEvent {
            event: Key(event), ..
        } => keybinding.handle_device_event(event),
        _ => (),
    });

    rsx! {
        document::Stylesheet {
            href: asset!("/assets/tailwind.css")
        },
        div {
            ..attributes,
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
