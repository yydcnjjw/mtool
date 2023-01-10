use dashmap::DashMap;
use std::{
    any::{type_name, Any, TypeId},
    ops::Deref,
};

type BoxedAny = Box<dyn Any + Send + Sync>;
type AnyMap = DashMap<TypeId, BoxedAny>;

#[derive(Debug)]
pub struct Container {
    inner: AnyMap,
}

impl Container {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }

    pub fn insert<T>(&self, v: T) -> Option<Box<T>>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.insert_any::<T>(Box::new(v))
    }

    pub fn insert_any<T>(&self, v: Box<dyn Any + Send + Sync>) -> Option<Box<T>>
    where
        T: Send + Sync + 'static,
    {
        log::debug!("insert {}", type_name::<T>());

        self.inner
            .insert(TypeId::of::<T>(), v)
            .and_then(|boxed| boxed.downcast().ok().map(|boxed| *boxed))
    }

    pub fn get<T>(&self) -> Option<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        log::debug!("get {}", type_name::<T>());

        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>().map(|v| v.clone()))
    }

    pub fn remove<T>(&self) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        self.inner
            .remove(&TypeId::of::<T>())
            .and_then(|(_, v)| v.downcast::<T>().map(|v| *v).ok())
    }

    pub fn contains_key<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.inner.contains_key(&TypeId::of::<T>())
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Container {
    type Target = AnyMap;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
