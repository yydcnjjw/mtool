use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

use async_trait::async_trait;
use minject::Provide;
use tokio::sync::{RwLock, RwLockMappedWriteGuard, RwLockReadGuard};

use crate::App;

use super::constructor::{Construct, IntoConstructor};

type BoxedConstruct = Box<dyn Construct<Injector> + Send + Sync>;

#[derive(Clone)]
pub struct Injector {
    inner: Arc<minject::Container>,
    constructor: Arc<RwLock<HashMap<TypeId, BoxedConstruct>>>,
}

impl Injector {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(minject::Container::new()),
            constructor: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn construct<Ctor, Args, Output>(&self, ctor: Ctor)
    where
        Ctor: IntoConstructor<Args, Output, Injector>,
        Ctor::Constructor: Construct<Injector> + Send + Sync + 'static,
        Output: Send + Sync + 'static,
    {
        self.constructor
            .write()
            .await
            .insert(TypeId::of::<Output>(), Box::new(ctor.into_constructor()));
    }

    pub async fn get<T>(&self) -> Option<RwLockReadGuard<T>>
    where
        T: Send + Sync + 'static,
    {
        log::debug!("get {}", type_name::<T>());

        let v = self.inner.get::<T>().await;

        if v.is_none() {
            let guard = self.constructor.read().await;
            let ctor = guard.get(&TypeId::of::<T>());

            match ctor?.construct(&self).await {
                Ok(v) => {
                    self.inner.insert_any::<T>(v).await;
                }
                Err(e) => {
                    log::error!("Failed to construct {}: {}", type_name::<T>(), e);
                }
            };
        };

        self.inner.get::<T>().await
    }

    pub async fn get_mut<T>(&self) -> Option<RwLockMappedWriteGuard<T>>
    where
        T: Send + Sync + 'static,
    {
        log::debug!("get_mut {}", type_name::<T>());

        let v = self.inner.get_mut::<T>().await;

        if v.is_none() {
            let guard = self.constructor.read().await;
            let ctor = guard.get(&TypeId::of::<T>());

            if let Ok(v) = ctor?.construct(&self).await {
                self.inner.insert_any::<T>(v).await;
            };
        };

        self.inner.get_mut::<T>().await
    }
}

impl Deref for Injector {
    type Target = minject::Container;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

#[async_trait]
impl Provide<App> for Injector {
    async fn provide(app: &App) -> Result<Self, anyhow::Error> {
        Ok(app.injector().clone())
    }
}
