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

impl LocalContainer {
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

    pub fn provide_value<T>(&mut self, value: Value)
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
}
