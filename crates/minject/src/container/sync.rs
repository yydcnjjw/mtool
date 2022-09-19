use std::{
    any::{Any, TypeId},
    collections::HashMap,
};
use tokio::sync::{RwLock, RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard};

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>>;

#[derive(Debug)]
pub struct Container {
    inner: RwLock<AnyMap>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub async fn insert<T>(&self, v: T) -> Option<Box<T>>
    where
        T: Send + Sync + 'static,
    {
        self.inner
            .write()
            .await
            .insert(TypeId::of::<T>(), Box::new(v))
            .and_then(|boxed| boxed.downcast().ok().map(|boxed| *boxed))
    }

    pub async fn insert_any<T>(&self, v: Box<dyn Any + Send + Sync>) -> Option<Box<T>>
    where
        T: Send + Sync + 'static,
    {
        self.inner
            .write()
            .await
            .insert(TypeId::of::<T>(), v)
            .and_then(|boxed| boxed.downcast().ok().map(|boxed| *boxed))
    }

    pub async fn get<T>(&self) -> Option<RwLockReadGuard<T>>
    where
        T: Send + Sync + 'static,
    {
        let guard = self.inner.read().await;
        RwLockReadGuard::try_map(guard, |v| {
            v.get(&TypeId::of::<T>()).and_then(|v| v.downcast_ref())
        })
        .ok()
    }

    pub async fn get_mut<T>(&self) -> Option<RwLockMappedWriteGuard<T>>
    where
        T: Send + Sync + 'static,
    {
        let guard = self.inner.write().await;
        RwLockWriteGuard::try_map(guard, |v| {
            v.get_mut(&TypeId::of::<T>()).and_then(|v| v.downcast_mut())
        })
        .ok()
    }

    pub async fn remove<T>(&self) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        self.inner
            .write()
            .await
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast().ok().map(|boxed| *boxed))
    }

    pub async fn clear(&self) {
        self.inner.write().await.clear()
    }

    pub async fn is_empty(&self) -> bool {
        self.inner.read().await.is_empty()
    }

    pub async fn len(&self) -> usize {
        self.inner.read().await.len()
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
