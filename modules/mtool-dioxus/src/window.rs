use dioxus::prelude::*;
use dioxus_desktop::{winit::window::Window, WindowAttributes};
use mapp::{
    anyhow,
    once_cell::sync::OnceCell,
    tokio::{self, sync::mpsc},
    tracing::warn,
};
use std::sync::Arc;

static SENDER: OnceCell<mpsc::UnboundedSender<AttachWebview>> = OnceCell::new();

pub fn use_window_factory() {
    use_hook(move || {
        spawn(async move {
            let (tx, mut rx) = mpsc::unbounded_channel();

            if let Err(e) = SENDER.set(tx) {
                warn!("{:?}", e);
            }

            while let Some(AttachWebview { window, app }) = rx.recv().await {
                let dom = VirtualDom::new(app);

                // TODO: rename attach webview
                dioxus_desktop::window().new_from_window(
                    dom,
                    dioxus_desktop::Config::new()
                        .with_as_child_window()
                        .with_menu(None)
                        .with_window(WindowAttributes::default().with_transparent(true)),
                    window,
                );
            }
        });
    });
}

struct AttachWebview {
    window: Arc<Window>,
    app: fn() -> Element,
}

impl std::fmt::Debug for AttachWebview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachWebview").finish()
    }
}

pub async fn attach_webview(
    window: Arc<Window>,
    app: fn() -> Element,
) -> Result<(), anyhow::Error> {
    let tx = match SENDER.get() {
        Some(tx) => tx,
        None => &tokio::task::spawn_blocking(move || SENDER.wait().clone()).await?,
    };

    Ok(tx.send(AttachWebview { window, app })?)
}
