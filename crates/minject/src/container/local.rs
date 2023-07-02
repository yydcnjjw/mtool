use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    fmt,
};

type LocalBoxedAny = Box<dyn Any>;
type LocalAnyMap = HashMap<TypeId, LocalBoxedAny>;

pub struct LocalContainer {
    inner: RefCell<LocalAnyMap>,
}

impl LocalContainer {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(HashMap::new()),
        }
    }

    pub fn insert<T>(&self, v: T) -> Option<Box<T>>
    where
        T: 'static,
    {
        self.insert_any(Box::new(v))
    }

    pub fn insert_any<T>(&self, v: LocalBoxedAny) -> Option<Box<T>>
    where
        T: 'static,
    {
        self.inner
            .borrow_mut()
            .insert(TypeId::of::<T>(), v)
            .and_then(|boxed| boxed.downcast().ok().map(|boxed| *boxed))
    }

    pub fn get<T>(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        self.inner
            .borrow()
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref())
            .cloned()
    }

    pub fn contains_key<T>(&self) -> bool
    where
        T: 'static,
    {
        self.inner.borrow().contains_key(&TypeId::of::<T>())
    }

    pub fn remove<T>(&self) -> Option<T>
    where
        T: 'static,
    {
        self.inner
            .borrow_mut()
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast().ok().map(|boxed| *boxed))
    }

    pub fn clear(&self) {
        self.inner.borrow_mut().clear()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.borrow().len()
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
