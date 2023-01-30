use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
};

type BoxedAny = Box<dyn Any + Send + Sync>;
type AnyMap = HashMap<TypeId, BoxedAny>;

pub struct LocalContainer {
    inner: AnyMap,
}

impl LocalContainer {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn insert<T>(&mut self, v: T) -> Option<Box<T>>
    where
        T: Send + Sync + 'static,
    {
        self.insert_any(Box::new(v))
    }

    pub fn insert_any<T>(&mut self, v: BoxedAny) -> Option<Box<T>>
    where
        T: Send + Sync + 'static,
    {
        self.inner
            .insert(TypeId::of::<T>(), v)
            .and_then(|boxed| boxed.downcast().ok().map(|boxed| *boxed))
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref())
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|v| v.downcast_mut())
    }

    pub fn contains_key<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        self.inner
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast().ok().map(|boxed| *boxed))
    }

    pub fn clear(&mut self) {
        self.inner.clear()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl Default for LocalContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for LocalContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalContainer").finish()
    }
}
