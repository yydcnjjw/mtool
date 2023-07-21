use std::{
    any::{type_name, TypeId},
    cell::RefCell,
    collections::HashMap,
    ops::Deref,
    rc::Rc,
    sync::Arc,
};

use async_trait::async_trait;
use dashmap::DashMap;
use minject::{LocalProvide, Provide};
use tokio::sync::{oneshot, Mutex};
use tracing::trace;

use crate::{App, LocalApp};

use super::{
    constructor::{Construct, IntoConstructor},
    BoxedAny, ConstructOnce, IntoLocalConstructor, IntoLocalOnceConstructor, IntoOnceConstructor,
    LocalBoxedAny, LocalConstruct, LocalConstructOnce, Res,
};

type BoxedConstruct = Box<dyn Construct<Injector> + Send + Sync>;
type BoxedConstructOnce = Box<dyn ConstructOnce<Injector> + Send + Sync>;

type LocalBoxedConstruct = Box<dyn LocalConstruct<LocalInjector>>;
type LocalBoxedConstructOnce = Box<dyn LocalConstructOnce<LocalInjector>>;

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

        self.inner.get::<T>(self).await
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
        Ctor::Constructor: Construct<Injector> + Send + Sync + 'static,
        Output: Send + Sync + 'static,
    {
        self.constructor
            .insert(TypeId::of::<Output>(), Box::new(ctor.into_constructor()));

        self
    }

    pub fn construct_once<Ctor, Args, Output>(&self, ctor: Ctor) -> &Self
    where
        Ctor: IntoOnceConstructor<Args, Output, Injector>,
        Ctor::OnceConstructor: ConstructOnce<Injector> + Send + Sync + 'static,
        Output: Send + Sync + 'static,
    {
        self.constructor_once.insert(
            TypeId::of::<Output>(),
            Box::new(ctor.into_once_constructor()),
        );

        self
    }

    pub fn construct_oneshot<Output>(&self) -> oneshot::Sender<Output>
    where
        Output: Send + Sync + 'static,
    {
        let (tx, rx) = oneshot::channel();
        self.construct_once(|| async move {
            Ok(Res::new(rx.await.map_err(|e| {
                anyhow::anyhow!("get {} failed: {}", type_name::<Output>(), e)
            })?))
        });

        tx
    }

    pub fn insert<T>(&self, v: T) -> Option<Box<T>>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.cont.insert(v)
    }

    pub async fn get<T>(&self, injector: &Injector) -> Result<T, anyhow::Error>
    where
        T: Send + Sync + Clone + 'static,
    {
        match self.cont.get::<T>() {
            Some(v) => Ok(v),
            None => {
                let v = self.construct_init::<T>(injector).await?;
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

    async fn construct_init<T>(&self, injector: &Injector) -> Result<BoxedAny, anyhow::Error>
    where
        T: Send + Sync + Clone + 'static,
    {
        let key = &TypeId::of::<T>();

        if let Some(ctor) = self.constructor.get(key) {
            trace!("try construct {}", type_name::<T>());

            let v = ctor.construct(injector).await;

            trace!("construct {}", type_name::<T>());

            v
        } else if let Some((_, ctor)) = self.constructor_once.remove(key) {
            trace!("try construct once {}", type_name::<T>());

            let v = ctor.construct_once(injector).await;

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

#[derive(Clone)]
pub struct LocalInjector {
    inner: Rc<LocalInjectorInner>,
    type_mutex: Rc<Mutex<HashMap<TypeId, Rc<Mutex<()>>>>>,
}

impl Deref for LocalInjector {
    type Target = LocalInjectorInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait(?Send)]
impl LocalProvide<LocalApp> for LocalInjector {
    async fn local_provide(app: &LocalApp) -> Result<Self, anyhow::Error> {
        Ok(app.injector().clone())
    }
}

#[async_trait(?Send)]
impl LocalProvide<LocalInjector> for LocalInjector {
    async fn local_provide(injector: &LocalInjector) -> Result<Self, anyhow::Error> {
        Ok(injector.clone())
    }
}

impl LocalInjector {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(LocalInjectorInner::new()),
            type_mutex: Rc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get<T>(&self) -> Result<T, anyhow::Error>
    where
        T: Clone + 'static,
    {
        let mutex = {
            let mut type_mutex = self.type_mutex.lock().await;
            type_mutex
                .entry(TypeId::of::<T>())
                .or_insert(Rc::new(Mutex::new(())))
                .clone()
        };

        let _guard = mutex.lock().await;

        self.inner.get::<T>(self).await
    }
}

pub struct LocalInjectorInner {
    cont: minject::LocalContainer,
    constructor: RefCell<HashMap<TypeId, LocalBoxedConstruct>>,
    constructor_once: RefCell<HashMap<TypeId, LocalBoxedConstructOnce>>,
}

impl LocalInjectorInner {
    pub fn new() -> Self {
        Self {
            cont: minject::LocalContainer::new(),
            constructor: RefCell::new(HashMap::new()),
            constructor_once: RefCell::new(HashMap::new()),
        }
    }

    pub fn construct<Ctor, Args, Output>(&self, ctor: Ctor) -> &Self
    where
        Ctor: IntoLocalConstructor<Args, Output, LocalInjector>,
        Ctor::LocalConstructor: LocalConstruct<LocalInjector> + 'static,
        Output: 'static,
    {
        self.constructor.borrow_mut().insert(
            TypeId::of::<Output>(),
            Box::new(ctor.into_local_constructor()),
        );

        self
    }

    pub fn construct_once<Ctor, Args, Output>(&self, ctor: Ctor) -> &Self
    where
        Ctor: IntoLocalOnceConstructor<Args, Output, LocalInjector>,
        Ctor::LocalOnceConstructor: LocalConstructOnce<LocalInjector> + 'static,
        Output: 'static,
    {
        self.constructor_once.borrow_mut().insert(
            TypeId::of::<Output>(),
            Box::new(ctor.into_local_once_constructor()),
        );

        self
    }

    pub fn construct_oneshot<Output>(&self) -> oneshot::Sender<Output>
    where
        Output: 'static,
    {
        let (tx, rx) = oneshot::channel();
        self.construct_once(|| async move {
            Ok(Res::new(rx.await.map_err(|e| {
                anyhow::anyhow!("get {} failed: {}", type_name::<Output>(), e)
            })?))
        });

        tx
    }

    pub fn insert<T>(&self, v: T) -> Option<Box<T>>
    where
        T: 'static,
    {
        self.cont.insert(v)
    }

    pub async fn get<T>(&self, injector: &LocalInjector) -> Result<T, anyhow::Error>
    where
        T: Clone + 'static,
    {
        match self.cont.get::<T>() {
            Some(v) => Ok(v),
            None => {
                let v = self.construct_init::<T>(injector).await?;
                let v = *v.downcast::<T>().unwrap();
                self.cont.insert(v);

                Ok(self.cont.get::<T>().unwrap())
            }
        }
    }

    pub async fn get_without_construct<T>(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        self.cont.get::<T>()
    }

    async fn construct_init<T>(
        &self,
        injector: &LocalInjector,
    ) -> Result<LocalBoxedAny, anyhow::Error>
    where
        T: 'static,
    {
        let key = &TypeId::of::<T>();

        if let Some(ctor) = self.constructor.borrow().get(key) {
            trace!("try local construct {}", type_name::<T>());

            let v = ctor.local_construct(injector).await;

            trace!("local construct {}", type_name::<T>());

            v
        } else if let Some(ctor) = self.constructor_once.borrow_mut().remove(key) {
            trace!("try local construct once {}", type_name::<T>());

            let v = ctor.local_construct_once(injector).await;

            trace!("local construct once {}", type_name::<T>());

            v
        } else {
            anyhow::bail!("{} is not exist", type_name::<T>())
        }
    }
}

impl Deref for LocalInjectorInner {
    type Target = minject::LocalContainer;

    fn deref(&self) -> &Self::Target {
        &self.cont
    }
}
