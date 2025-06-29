use bevy::log::warn;
use dioxus::prelude::*;
use mapp::{sync::RwLock, tracing::debug};
use std::{collections::HashMap, sync::Arc};

pub type RouteParams = HashMap<String, String>;
pub type RouteHandler = Arc<dyn Fn(&RouteParams) -> Element + Send + Sync>;

#[derive(Clone)]
pub struct Router {
    inner: Arc<RwLock<RouterInner>>,
}

pub struct RouterInner {
    route: Option<(RouteParams, RouteHandler)>,
    recognizer: route_recognizer::Router<RouteHandler>,
}
impl RouterInner {
    pub fn new() -> Self {
        Self {
            route: None,
            recognizer: route_recognizer::Router::new(),
        }
    }

    pub fn add<Handler>(&mut self, route: &str, handler: Handler)
    where
        Handler: Fn(&RouteParams) -> Element + Send + Sync + 'static,
    {
        debug!("add route {}", route);
        self.recognizer.add(route, Arc::new(handler));
    }

    pub fn route(&mut self, path: &str) {
        match self.recognize(path) {
            Ok(route) => {
                debug!("route is changed: {}", path);
                self.route = Some(route);
            }
            Err(e) => {
                warn!("{}", e)
            }
        }
    }

    fn recognize(&self, path: &str) -> Result<(RouteParams, RouteHandler), String> {
        self.recognizer.recognize(path).map(|v| {
            (
                v.params()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                (*v.handler()).clone(),
            )
        })
    }
}

#[macro_export]
macro_rules! add_route {
    ($router:expr, $path:expr, $component:path) => {
        $router.add($path, |params| -> Element {
            // use_context_provider({to_owned![params]; move || params});
            rsx! {
                $component {}
            }
        })
    };
}


impl Router {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RouterInner::new())),
        }
    }

    pub fn add<Handler>(&self, route: &str, handler: Handler) -> &Self
    where
        Handler: Fn(&RouteParams) -> Element + Send + Sync + 'static,
    {
        self.inner.write().add(route, handler);
        &self
    }

    pub fn route(&self, path: &str) {
        self.inner.write().route(path);
    }

    pub fn render(&self) -> Element {
        if let Some((param, handler)) = &self.inner.read().route {
            handler(param)
        } else {
            rsx! {
                div {
                    "no route"
                }
            }
        }
    }
}

#[derive(Routable, Clone)]
pub enum Route {
    #[route("/:path")]
    Main { path: String },

    #[route("/")]
    Default {},
}

pub fn app_route(path: &str) -> Route {
    Route::Main {
        path: path.to_string(),
    }
}

#[component]
fn Main(path: String) -> Element {
    let router = use_context::<Router>();
    router.route(&path);
    router.render()
}

#[component]
fn Default() -> Element {
    let router = use_context::<Router>();
    router.render()
}
