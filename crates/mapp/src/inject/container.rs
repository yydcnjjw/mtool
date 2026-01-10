use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
};

use tracing::warn;

use crate::lazy::LazyCell;

type LocalBoxAny = Box<dyn Any>;

enum Value {
    Lazy(LazyCell<LocalBoxAny>),
    Value(LocalBoxAny),
}

pub struct LocalContainer {
    inner: HashMap<TypeId, Value>,
}

impl Default for LocalContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalContainer {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn lazy_provide<T>(&mut self, value: LazyCell<LocalBoxAny>)
    where
        T: 'static,
    {
        self.provide_value::<T>(Value::Lazy(value));
    }

    pub fn provide<T>(&mut self, value: T)
    where
        T: 'static,
    {
        self.provide_value::<T>(Value::Value(Box::new(value)));
    }

    fn provide_value<T>(&mut self, value: Value)
    where
        T: 'static,
    {
        if let Some(_) = self.inner.insert(TypeId::of::<T>(), value) {
            warn!("{} is replaced", type_name::<T>())
        }
    }

    pub fn try_consume<T>(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|value| match value {
                Value::Lazy(_) => None,
                Value::Value(value) => value.downcast_ref().cloned(),
            })
    }

    pub async fn consume<T>(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        match self.inner.get(&TypeId::of::<T>())? {
            Value::Lazy(lazy) => LazyCell::force(&lazy).await.downcast_ref().cloned(),
            Value::Value(value) => value.downcast_ref().cloned(),
        }
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        match self.inner.remove(&TypeId::of::<T>())? {
            Value::Lazy(_) => None,
            Value::Value(value) => value.downcast().map(|v| *v).ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provide_and_consume() {
        // Test scenario: Directly provide a value, then attempt to
        // synchronously consume and asynchronously consume that
        // value.
        let mut container = LocalContainer::new();
        container.provide::<i32>(42);

        assert_eq!(container.try_consume::<i32>(), Some(42));
        assert_eq!(container.consume::<i32>().await, Some(42));
    }

    #[tokio::test]
    async fn test_lazy_provide_and_consume() {
        // Test scenario: Provide a lazily initialized value, verify
        // that try_consume cannot obtain it, but consume can obtain
        // it and trigger initialization.
        let mut container = LocalContainer::new();
        let value = 42;
        container.lazy_provide::<i32>(LazyCell::new(move || {
            Box::pin(async move { Box::new(value) as LocalBoxAny })
        }));

        assert_eq!(container.try_consume::<i32>(), None);
        assert_eq!(container.consume::<i32>().await, Some(42));

        // Again consume, verify value whether maintain consistency
        assert_eq!(container.consume::<i32>().await, Some(42));
    }

    #[tokio::test]
    async fn test_replace_value() {
        // Test scenario: Provide a value and then provide another
        // value of the same type, verify that the old value is
        // replaced.
        let mut container = LocalContainer::new();
        container.provide::<i32>(10);
        assert_eq!(container.consume::<i32>().await, Some(10));

        container.provide::<i32>(20);
        assert_eq!(container.consume::<i32>().await, Some(20));
    }

    #[tokio::test]
    async fn test_remove_value() {
        // Test scenario: Provide a value then remove it, verify
        // removal success and inability to retrieve again.
        let mut container = LocalContainer::new();
        container.provide::<String>("test".to_string());

        assert_eq!(container.remove::<String>(), Some("test".to_string()));
        assert_eq!(container.consume::<String>().await, None);
    }

    #[tokio::test]
    async fn test_remove_lazy_value() {
        // Test scenario: Attempt to remove a lazy value, expected to
        // be unable to remove (return None), but the value is indeed
        // removed from the container.
        let mut container = LocalContainer::new();
        container.lazy_provide::<i32>(LazyCell::new(|| {
            Box::pin(async { Box::new(100) as LocalBoxAny })
        }));

        // remove Current implementation for Lazy returns None
        assert_eq!(container.remove::<i32>(), None);

        // Verify value has been removed
        assert_eq!(container.consume::<i32>().await, None);
    }
}
