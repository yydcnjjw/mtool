use dioxus::prelude::*;
use dioxus_desktop::{winit::window::Window, Config};
use mapp::{
    anyhow,
    once_cell::sync::OnceCell,
    tokio::{self, sync::mpsc},
    tracing::warn,
};
use std::{any::Any, sync::Arc};

static SENDER: OnceCell<mpsc::UnboundedSender<WinitWebviewBuilder>> = OnceCell::new();

pub fn use_window_factory() {
    use_hook(move || {
        spawn(async move {
            let (tx, mut rx) = mpsc::unbounded_channel();

            if let Err(e) = SENDER.set(tx) {
                warn!("{:?}", e);
            }

            while let Some(WinitWebviewBuilder {
                window,
                app,
                contexts,
                as_child_window,
            }) = rx.recv().await
            {
                let mut dom = VirtualDom::new(app);
                for ctx in contexts {
                    dom.insert_any_root_context(ctx());
                }

                let mut config = dioxus_desktop::Config::default().with_menu(None);

                if as_child_window {
                    config = config.with_as_child_window();
                }

                // TODO: rename attach webview
                dioxus_desktop::window().new_from_window(dom, config, window);
            }
        });
    });
}

type ContextFn = Box<dyn Fn() -> Box<dyn Any> + Send + Sync + 'static>;

pub struct WinitWebviewBuilder {
    window: Arc<Window>,
    app: fn() -> Element,
    contexts: Vec<ContextFn>,
    as_child_window: bool,
}

impl WinitWebviewBuilder {
    pub fn new(window: Arc<Window>, app: fn() -> Element) -> Self {
        Self {
            window,
            app,
            contexts: Vec::new(),
            as_child_window: false,
        }
    }

    pub fn with_context(mut self, state: impl Any + Clone + Send + Sync + 'static) -> Self {
        self.contexts
            .push(Box::new(move || Box::new(state.clone())));
        self
    }

    pub fn with_as_child_window(mut self) -> Self {
        self.as_child_window = true;
        self
    }

    pub async fn build(self) -> Result<(), anyhow::Error> {
        let tx = match SENDER.get() {
            Some(tx) => tx,
            None => &tokio::task::spawn_blocking(move || SENDER.wait().clone()).await?,
        };

        Ok(tx.send(self)?)
    }
}
