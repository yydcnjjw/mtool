use std::{convert::Infallible, ops::DerefMut};

use mapp::sync::Mutex;
use tonic::{
    body::Body,
    server::NamedService,
    service::{Routes, RoutesBuilder},
};
use tower::Service;

pub struct Router {
    builder: Mutex<RoutesBuilder>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            builder: Mutex::new(RoutesBuilder::default()),
        }
    }

    pub fn add_service<S>(&self, svc: S) -> &Self
    where
        S: Service<axum::http::Request<Body>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Response: axum::response::IntoResponse,
        S::Future: Send + 'static,
    {
        self.builder.lock().add_service(svc);
        self
    }

    pub(crate) fn routes(self) -> Routes {
        let mut builder = self.builder.lock();
        std::mem::take(builder.deref_mut()).routes()
    }
}
