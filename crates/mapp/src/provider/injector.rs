use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

use async_trait::async_trait;
use dashmap::DashMap;
use minject::Provide;
use tokio::sync::Mutex;
use tracing::trace;

use crate::App;

use super::{
    constructor::{Construct, IntoConstructor},
    BoxedAny, ConstructOnce, IntoOnceConstructor,
};

type BoxedConstruct = Box<dyn Construct<InjectorInner> + Send + Sync>;
type BoxedConstructOnce = Box<dyn ConstructOnce<InjectorInner> + Send + Sync>;

#[derive(Clone)]
pub struct Injector {
    inner: Arc<InjectorInner>,
    type_mutex: Arc<Mutex<HashMap<TypeId, Arc<Mutex<()>>>>>,
}

impl Deref for Injector {
    type Target = InjectorInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait]
impl Provide<App> for Injector {
    async fn provide(app: &App) -> Result<Self, anyhow::Error> {
        Ok(app.injector().clone())
    }
}

#[async_trait]
impl Provide<Injector> for Injector {
    async fn provide(injector: &Injector) -> Result<Self, anyhow::Error> {
        Ok(injector.clone())
    }
}

impl Injector {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InjectorInner::new()),
            type_mutex: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get<T>(&self) -> Result<T, anyhow::Error>
    where
        T: Send + Sync + Clone + 'static,
    {
        let mutex = {
            let mut type_mutex = self.type_mutex.lock().await;
            type_mutex
                .entry(TypeId::of::<T>())
                .or_insert(Arc::new(Mutex::new(())))
                .clone()
        };

        let _guard = mutex.lock().await;

        self.inner.get::<T>().await
    }
}

pub struct InjectorInner {
    cont: minject::Container,
    constructor: DashMap<TypeId, BoxedConstruct>,
    constructor_once: DashMap<TypeId, BoxedConstructOnce>,
}

impl InjectorInner {
    pub fn new() -> Self {
        Self {
            cont: minject::Container::new(),
            constructor: DashMap::new(),
            constructor_once: DashMap::new(),
        }
    }

    pub fn construct<Ctor, Args, Output>(&self, ctor: Ctor) -> &Self
    where
        Ctor: IntoConstructor<Args, Output, Injector>,
        Ctor::Constructor: Construct<InjectorInner> + Send + Sync + 'static,
        Output: Send + Sync + 'static,
    {
        self.constructor
            .insert(TypeId::of::<Output>(), Box::new(ctor.into_constructor()));

        self
    }

    pub fn construct_once<Ctor, Args, Output>(&self, ctor: Ctor) -> &Self
    where
        Ctor: IntoOnceConstructor<Args, Output, Injector>,
        Ctor::OnceConstructor: ConstructOnce<InjectorInner> + Send + Sync + 'static,
        Output: Send + Sync + 'static,
    {
        self.constructor_once.insert(
            TypeId::of::<Output>(),
            Box::new(ctor.into_once_constructor()),
        );

        self
    }

    pub fn insert<T>(&self, v: T) -> Option<Box<T>>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.cont.insert(v)
    }

    pub async fn get<T>(&self) -> Result<T, anyhow::Error>
    where
        T: Send + Sync + Clone + 'static,
    {
        match self.cont.get::<T>() {
            Some(v) => Ok(v),
            None => {
                let v = self.construct_init::<T>().await?;
                let v = *v.downcast::<T>().unwrap();
                self.cont.insert(v.clone());
                Ok(v)
            }
        }
    }

    pub async fn get_without_construct<T>(&self) -> Option<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.cont.get::<T>()
    }

    async fn construct_init<T>(&self) -> Result<BoxedAny, anyhow::Error>
    where
        T: Send + Sync + Clone + 'static,
    {
        let key = &TypeId::of::<T>();

        if let Some(ctor) = self.constructor.get(key) {
            trace!("try construct {}", type_name::<T>());

            let v = ctor.construct(self).await;

            trace!("construct {}", type_name::<T>());

            v
        } else if let Some((_, ctor)) = self.constructor_once.remove(key) {
            trace!("try construct once {}", type_name::<T>());

            let v = ctor.construct_once(self).await;

            trace!("construct once {}", type_name::<T>());

            v
        } else {
            anyhow::bail!("{} is not exist", type_name::<T>())
        }
    }
}

impl Deref for InjectorInner {
    type Target = minject::Container;

    fn deref(&self) -> &Self::Target {
        &self.cont
    }
}
