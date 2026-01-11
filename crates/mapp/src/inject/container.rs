use std::{
    any::{Any, TypeId, type_name},
    cell::RefCell,
    collections::HashMap,
};

use tracing::warn;

use crate::lazy::LazyCell;

type LocalBoxAny = Box<dyn Any>;

enum Value {
    Lazy(LazyCell<LocalBoxAny>),
    Value(LocalBoxAny),
}

pub struct LocalTypedMap {
    inner: RefCell<HashMap<TypeId, Value>>,
}

impl Default for LocalTypedMap {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalTypedMap {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(HashMap::new()),
        }
    }

    pub fn insert_lazy<F, Fut, T>(&self, f: F)
    where
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = T> + 'static,
        T: 'static,
    {
        self.insert_value::<T>(Value::Lazy(LazyCell::new(move || {
            Box::pin(async move { Box::new(f().await) as LocalBoxAny })
        })));
    }

    pub fn insert<T>(&self, value: T)
    where
        T: 'static,
    {
        self.insert_value::<T>(Value::Value(Box::new(value)));
    }

    fn insert_value<T>(&self, value: Value)
    where
        T: 'static,
    {
        if let Some(_) = self.inner.borrow_mut().insert(TypeId::of::<T>(), value) {
            warn!("{} is replaced", type_name::<T>())
        }
    }

    pub fn try_get<T>(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        self.inner
            .borrow()
            .get(&TypeId::of::<T>())
            .and_then(|value| match value {
                Value::Lazy(_) => None,
                Value::Value(value) => value.downcast_ref().cloned(),
            })
    }

    pub async fn get<T>(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        match self.inner.borrow().get(&TypeId::of::<T>())? {
            Value::Lazy(lazy) => LazyCell::force(&lazy).await,
            Value::Value(value) => value,
        }
        .downcast_ref()
        .cloned()
    }

    pub async fn take<T: 'static>(&self) -> Option<T> {
        match self.inner.borrow_mut().remove(&TypeId::of::<T>())? {
            Value::Lazy(lazy) => {
                _ = LazyCell::force(&lazy).await;
                LazyCell::into_inner(lazy)
                    .map_err(|_| "")
                    .expect("LazyCell initialized")
            }
            Value::Value(value) => value,
        }
        .downcast()
        .map(|value| *value)
        .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_and_get() {
        // Test scenario: Directly provide a value, then attempt to
        // synchronously consume and asynchronously consume that
        // value.
        let container = LocalTypedMap::new();
        container.insert::<i32>(42);

        assert_eq!(container.try_get::<i32>(), Some(42));
        assert_eq!(container.get::<i32>().await, Some(42));
    }

    #[tokio::test]
    async fn test_insert_lazy_and_get() {
        // Test scenario: Provide a lazily initialized value, verify
        // that try_consume cannot obtain it, but consume can obtain
        // it and trigger initialization.
        let container = LocalTypedMap::new();
        let value = 42;
        container.insert_lazy(async move || value);

        assert_eq!(container.try_get::<i32>(), None);
        assert_eq!(container.get::<i32>().await, Some(42));

        // Again consume, verify value whether maintain consistency
        assert_eq!(container.get::<i32>().await, Some(42));
    }

    #[tokio::test]
    async fn test_replace_value() {
        // Test scenario: Provide a value and then provide another
        // value of the same type, verify that the old value is
        // replaced.
        let container = LocalTypedMap::new();
        container.insert::<i32>(10);
        assert_eq!(container.get::<i32>().await, Some(10));

        container.insert::<i32>(20);
        assert_eq!(container.get::<i32>().await, Some(20));
    }

    #[tokio::test]
    async fn test_take_value() {
        // Test scenario: Provide a value then remove it, verify
        // removal success and inability to retrieve again.
        let container = LocalTypedMap::new();
        container.insert::<String>("test".to_string());

        assert_eq!(container.take::<String>().await, Some("test".to_string()));
        assert_eq!(container.get::<String>().await, None);
    }

    #[tokio::test]
    async fn test_take_lazy_value() {
        // Test scenario: Attempt to remove a lazy value, expected to
        // be unable to remove (return None), but the value is indeed
        // removed from the container.
        let container = LocalTypedMap::new();
        container.insert_lazy(async move || 100);

        // remove Current implementation for Lazy returns None
        assert_eq!(container.take::<i32>().await, Some(100));

        // Verify value has been removed
        assert_eq!(container.get::<i32>().await, None);
    }
}
