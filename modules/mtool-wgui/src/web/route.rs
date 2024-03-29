use std::{cell::RefCell, collections::HashMap, rc::Rc, fmt};

use tracing::debug;

use yew::prelude::*;
use yew_router::Routable;

pub type RouteParams = HashMap<String, String>;
pub type RouteHandler = Rc<dyn Fn(&RouteParams) -> Html>;

#[derive(Clone)]
pub struct Router {
    inner: Rc<RefCell<route_recognizer::Router<RouteHandler>>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(route_recognizer::Router::new())),
        }
    }

    pub fn add<Handler>(&self, route: &str, handler: Handler) -> &Self
    where
        Handler: Fn(&RouteParams) -> Html + 'static,
    {
        debug!("add route {}", route);
        self.inner.borrow_mut().add(route, Rc::new(handler));
        self
    }

    pub fn recognize(&self, path: &str) -> Result<(RouteParams, RouteHandler), String> {
        self.inner.borrow().recognize(path).map(|v| {
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

pub(crate) fn global_router() -> Router {
    thread_local! {
        static INST: Router = Router::new();
    }

    INST.with(|v| v.clone())
}

#[derive(Clone)]
pub struct Route {
    path: String,
    params: RouteParams,
    handler: Option<RouteHandler>,
}

impl Route {
    pub fn call(&self) -> Html {
        if let Some(handler) = &self.handler {
            (*handler)(&self.params)
        } else {
            html! {}
        }
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route").finish()
    }
}

impl Eq for Route {}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.params == other.params
    }
}

impl Routable for Route {
    fn from_path(path: &str, params: &std::collections::HashMap<&str, &str>) -> Option<Self> {
        let router = global_router();
        let (_, handler) = router.recognize(path).ok()?;

        Some(Self {
            path: path.to_string(),
            params: params
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            handler: Some(handler),
        })
    }

    fn to_path(&self) -> String {
        self.path.clone()
    }

    fn routes() -> Vec<&'static str> {
        unimplemented!()
    }

    fn not_found_route() -> Option<Self> {
        None
    }

    fn recognize(pathname: &str) -> Option<Self> {
        debug!("recognize: {}", pathname);

        global_router()
            .recognize(pathname)
            .map(|(params, handler)| Self {
                path: pathname.to_string(),
                params,
                handler: Some(handler),
            })
            .ok()
    }
}
