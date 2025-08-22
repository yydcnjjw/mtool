use dioxus::prelude::*;
use dioxus_desktop::{
    winit::window::Window, wry::http, wry::WebViewId, Config, RequestAsyncResponder,
    WindowAttributes, WindowCloseBehaviour,
};
use mapp::{
    anyhow,
    once_cell::sync::OnceCell,
    prelude::Injector,
    tokio::{self, sync::mpsc},
    tracing::warn,
};
use std::{any::Any, borrow::Cow, path::PathBuf, sync::Arc};

use crate::context::DioxusContext;

static SENDER: OnceCell<mpsc::UnboundedSender<WebviewWindowConfig>> = OnceCell::new();

pub fn use_window_factory() {
    let injector = use_context::<Injector>();
    let dioxus_context = use_context::<DioxusContext>();

    use_hook(move || {
        to_owned![injector, dioxus_context];

        let (tx, mut rx) = mpsc::unbounded_channel();

        if let Err(e) = SENDER.set(tx) {
            warn!("{:?}", e);
        }

        spawn(async move {
            while let Some(config) = rx.recv().await {
                match config.try_into() {
                    Ok((mut dom, config, window)) => {
                        dom.insert_any_root_context(Box::new(injector.clone()));
                        dom.insert_any_root_context(Box::new(dioxus_context.clone()));

                        match window {
                            Some(window) => {
                                dioxus_desktop::window().new_from_window(dom, config, window);
                            }
                            None => {
                                dioxus_desktop::window().new_window(dom, config);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("{e:?}");
                    }
                }
            }
        });
    });
}

type ContextFn = Box<dyn Fn() -> Box<dyn Any> + Send + Sync + 'static>;

type WryProtocol = (
    String,
    Box<
        dyn Fn(WebViewId, http::Request<Vec<u8>>) -> http::Response<Cow<'static, [u8]>>
            + Send
            + Sync
            + 'static,
    >,
);

type AsyncWryProtocol = (
    String,
    Box<dyn Fn(WebViewId, http::Request<Vec<u8>>, RequestAsyncResponder) + Send + Sync + 'static>,
);

pub struct WebviewWindowConfig {
    app: fn() -> Element,

    window: Option<Arc<Window>>,
    contexts: Vec<ContextFn>,

    window_attributes: WindowAttributes,
    as_child_window: bool,
    protocols: Vec<WryProtocol>,
    asynchronous_protocols: Vec<AsyncWryProtocol>,
    pre_rendered: Option<String>,
    disable_context_menu: bool,
    resource_dir: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    custom_head: Option<String>,
    custom_index: Option<String>,
    root_name: String,
    background_color: Option<(u8, u8, u8, u8)>,
    exit_on_last_window_close: bool,
    window_close_behavior: WindowCloseBehaviour,
    disable_file_drop_handler: bool,
}

impl TryFrom<WebviewWindowConfig> for (VirtualDom, Config, Option<Arc<Window>>) {
    type Error = anyhow::Error;

    fn try_from(
        WebviewWindowConfig {
            app,
            window,
            contexts,
            window_attributes,
            as_child_window,
            protocols,
            asynchronous_protocols,
            pre_rendered,
            disable_context_menu,
            resource_dir,
            data_dir,
            custom_head,
            custom_index,
            root_name,
            background_color,
            exit_on_last_window_close,
            window_close_behavior,
            disable_file_drop_handler,
        }: WebviewWindowConfig,
    ) -> Result<Self, Self::Error> {
        let mut dom = VirtualDom::new(app);

        for ctx in contexts {
            dom.insert_any_root_context(ctx());
        }

        let mut config = dioxus_desktop::Config::default()
            .with_menu(None)
            .with_window(window_attributes)
            .with_disable_context_menu(disable_context_menu)
            .with_root_name(root_name)
            .with_exits_when_last_window_closes(exit_on_last_window_close)
            .with_close_behaviour(window_close_behavior)
            .with_disable_drag_drop_handler(disable_file_drop_handler);

        for (name, handler) in protocols {
            config = config.with_custom_protocol(name, handler);
        }

        for (name, handler) in asynchronous_protocols {
            config = config.with_asynchronous_custom_protocol(name, handler);
        }

        if let Some(pre_rendered) = pre_rendered {
            config = config.with_prerendered(pre_rendered);
        }

        if let Some(resource_dir) = resource_dir {
            config = config.with_resource_directory(resource_dir);
        }

        if let Some(data_dir) = data_dir {
            config = config.with_data_directory(data_dir);
        }

        if let Some(custom_head) = custom_head {
            config = config.with_custom_head(custom_head);
        }

        if let Some(custom_index) = custom_index {
            config = config.with_custom_index(custom_index);
        }

        if let Some(background_color) = background_color {
            config = config.with_background_color(background_color);
        }

        if as_child_window {
            config = config.with_as_child_window();
        }

        Ok((dom, config, window))
    }
}

impl WebviewWindowConfig {
    pub fn new(app: fn() -> Element) -> Self {
        Self {
            app,
            window: None,
            contexts: Vec::new(),
            window_attributes: WindowAttributes::default(),
            as_child_window: false,
            protocols: Vec::new(),
            asynchronous_protocols: Vec::new(),
            pre_rendered: None,
            disable_context_menu: !cfg!(debug_assertions),
            resource_dir: None,
            data_dir: None,
            custom_head: None,
            custom_index: None,
            root_name: "main".to_string(),
            background_color: None,
            exit_on_last_window_close: true,
            window_close_behavior: WindowCloseBehaviour::WindowCloses,
            disable_file_drop_handler: false,
        }
    }

    pub fn with_window(mut self, window: Arc<Window>) -> Self {
        self.window = Some(window);
        self
    }

    pub fn with_context(mut self, state: impl Any + Clone + Send + Sync + 'static) -> Self {
        self.contexts
            .push(Box::new(move || Box::new(state.clone())));
        self
    }

    pub fn with_resource_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.resource_dir = Some(path.into());
        self
    }

    pub fn with_data_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    pub fn with_disable_context_menu(mut self, disable: bool) -> Self {
        self.disable_context_menu = disable;
        self
    }

    pub fn with_disable_drag_drop_handler(mut self, disable: bool) -> Self {
        self.disable_file_drop_handler = disable;
        self
    }

    pub fn with_prerendered(mut self, content: String) -> Self {
        self.pre_rendered = Some(content);
        self
    }

    pub fn with_window_attributes(mut self, attributes: WindowAttributes) -> Self {
        self.window_attributes = attributes;
        self
    }

    pub fn with_as_child_window(mut self) -> Self {
        self.as_child_window = true;
        self
    }

    pub fn with_exits_when_last_window_closes(mut self, exit: bool) -> Self {
        self.exit_on_last_window_close = exit;
        self
    }

    pub fn with_close_behaviour(mut self, behaviour: WindowCloseBehaviour) -> Self {
        self.window_close_behavior = behaviour;
        self
    }

    pub fn with_custom_protocol<F>(mut self, name: impl ToString, handler: F) -> Self
    where
        F: Fn(WebViewId, http::Request<Vec<u8>>) -> http::Response<Cow<'static, [u8]>>
            + Send
            + Sync
            + 'static,
    {
        self.protocols.push((name.to_string(), Box::new(handler)));
        self
    }

    pub fn with_asynchronous_custom_protocol<F>(mut self, name: impl ToString, handler: F) -> Self
    where
        F: Fn(WebViewId, http::Request<Vec<u8>>, RequestAsyncResponder) + Send + Sync + 'static,
    {
        self.asynchronous_protocols
            .push((name.to_string(), Box::new(handler)));
        self
    }

    pub fn with_custom_head(mut self, head: String) -> Self {
        self.custom_head = Some(head);
        self
    }

    pub fn with_custom_index(mut self, index: String) -> Self {
        self.custom_index = Some(index);
        self
    }

    pub fn with_root_name(mut self, name: impl Into<String>) -> Self {
        self.root_name = name.into();
        self
    }

    pub fn with_background_color(mut self, color: (u8, u8, u8, u8)) -> Self {
        self.background_color = Some(color);
        self
    }
}

pub async fn spawn_window(config: WebviewWindowConfig) -> Result<(), anyhow::Error> {
    let tx = match SENDER.get() {
        Some(tx) => tx,
        None => &tokio::task::spawn_blocking(move || SENDER.wait().clone()).await?,
    };

    Ok(tx.send(config)?)
}
