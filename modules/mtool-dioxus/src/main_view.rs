use dioxus::{prelude::*, CapturedError};
use dioxus_desktop::{
    trayicon::{init_tray_icon, menu::MenuItem, DioxusTrayMenu},
    use_global_shortcut, use_wry_event_handler, winit::event::Event,
    UserWindowEvent,
};
use mapp::{
    anyhow::{self, anyhow},
    tracing::warn,
};

use crate::{
    builder::GlobalHotkeys, keybinding::Keybinding, router::Route, window::use_window_factory,
};

fn provide_keybinding() -> Keybinding {
    let keybinding = Keybinding::new();
    {
        let keybinding = keybinding.clone();
        spawn(async move { keybinding.run_loop().await });
    }
    keybinding
}

fn init_global_hotkey() -> Result<(), anyhow::Error> {
    let hotkeys: GlobalHotkeys = use_context();

    for (key, cb) in hotkeys.iter() {
        to_owned![cb];
        use_global_shortcut(key.as_str(), move || {
            if let Err(e) = cb() {
                warn!("{:?}", e);
            }
        })
        .map_err(|e| anyhow!("register {} failed {:?}", key, e))?;
    }
    Ok(())
}

fn init_tray() {
    let quit = MenuItem::new("Quit", true, None);
    let tray_menu = DioxusTrayMenu::with_items(&[&quit]).unwrap();
    init_tray_icon(tray_menu, None);

    use_wry_event_handler(move |ev, event_loop| {
        if let Event::UserEvent(UserWindowEvent::TrayMenuEvent(ev)) = ev {
            let id = &ev.id;
            if id == quit.id() {
                event_loop.exit();
            }
        }
    });
}

pub fn main_view() -> Element {
    let keybinding = use_context_provider(provide_keybinding);

    let onkeydown = move |e| {
        keybinding.handle_web_key_down(e);
    };

    init_global_hotkey().map_err(|e| RenderError::Aborted(CapturedError::from_display(e)))?;

    use_window_factory();

    init_tray();

    rsx! {
        document::Stylesheet {
            href: asset!("/assets/tailwind.css")
        },
        body {
            onkeydown,
            Router::<Route> {}
        }
    }
}
