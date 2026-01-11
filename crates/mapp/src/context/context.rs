use futures::{FutureExt, TryFutureExt, future::LocalBoxFuture};
use minject::LocalProvide;
use snafu::ResultExt;
use std::{any::type_name, rc::Rc, sync::Arc};
use tokio::sync::oneshot;

use crate::inject::{self, LocalTypedMap, Take};

use super::{ContextError, ProvideError};

pub struct Context {
    local_injector: inject::LocalTypedMap,
}

#[derive(Debug, Clone)]
enum Value<T> {
    Value(T),
    Lazy(Result<T, ProvideError>),
}

impl Context {
    pub fn new() -> Self {
        Self {
            local_injector: LocalTypedMap::default(),
        }
    }

    pub async fn get_value<T>(&self) -> Result<T, ContextError>
    where
        T: Clone + 'static,
    {
        match self.local_injector.get::<Value<T>>().await.ok_or_else(|| {
            ContextError::ProvideNotFound {
                value: type_name::<T>(),
            }
        })? {
            Value::Value(value) => Ok(value),
            Value::Lazy(value) => {
                value.with_whatever_context(|_| format!("get value {}", type_name::<T>()))
            }
        }
    }

    pub async fn take_value<T>(&self) -> Result<T, ContextError>
    where
        T: 'static,
    {
        match self
            .local_injector
            .take::<Value<T>>()
            .await
            .ok_or_else(|| ContextError::ProvideNotFound {
                value: type_name::<T>(),
            })? {
            Value::Value(value) => Ok(value),
            Value::Lazy(value) => {
                value.with_whatever_context(|_| format!("get value {}", type_name::<T>()))
            }
        }
    }

    pub fn provide_value<T>(&self, value: T)
    where
        T: 'static,
    {
        self.local_injector.insert(Value::Value(value))
    }

    pub fn provide_lazy<F, Fut, T>(&self, f: F)
    where
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<T, ProvideError>> + 'static,
        T: 'static,
    {
        self.local_injector
            .insert_lazy(move || async move { Value::Lazy(f().await) })
    }

    pub fn provide_oneshot<T>(&self) -> oneshot::Sender<T>
    where
        T: 'static,
    {
        let (tx, rx) = oneshot::channel();

        self.local_injector.insert_lazy(move || async move {
            Value::Lazy(rx.await.map_err(|e| ProvideError::Whatever {
                message: format!("provide oneshot error: {}", e),
                source: Some(Rc::new(e)),
            }))
        });

        tx
    }
}

impl<T> LocalProvide<Rc<T>> for Context
where
    T: 'static,
{
    type Error = ContextError;

    fn local_provide(&self) -> LocalBoxFuture<'_, Result<Rc<T>, Self::Error>> {
        self.get_value::<Rc<T>>().boxed_local()
    }
}

impl<T> LocalProvide<Arc<T>> for Context
where
    T: 'static,
{
    type Error = ContextError;

    fn local_provide(&self) -> LocalBoxFuture<'_, Result<Arc<T>, Self::Error>> {
        self.get_value::<Arc<T>>().boxed_local()
    }
}

impl<T> LocalProvide<Take<T>> for Context
where
    T: 'static,
{
    type Error = ContextError;

    fn local_provide(&self) -> LocalBoxFuture<'_, Result<Take<T>, Self::Error>> {
        self.take_value::<T>()
            .map_ok(|value| Take::new(value))
            .boxed_local()
    }
}

impl<T> LocalProvide<Option<T>> for Context
where
    T: 'static,
    Context: LocalProvide<T>,
{
    type Error = <Context as LocalProvide<T>>::Error;

    fn local_provide(&self) -> LocalBoxFuture<'_, Result<Option<T>, Self::Error>> {
        async { Ok(LocalProvide::<T>::local_provide(self).await.ok()) }.boxed_local()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provide_value_and_get_value() {
        let context = Context::new();

        context.provide_value(42);
        assert_eq!(context.get_value::<i32>().await.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_provide_lazy_and_get_value() {
        let context = Context::new();

        context.provide_lazy(|| async { Ok::<i32, ProvideError>(42) });
        assert_eq!(context.get_value::<i32>().await.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_provide_oneshot() {
        let context = Context::new();

        let tx = context.provide_oneshot::<i32>();
        tokio::spawn(async move {
            tx.send(42).unwrap();
        });
        assert_eq!(context.get_value::<i32>().await.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_take_value() {
        let context = Context::new();

        context.provide_value(42);
        assert_eq!(context.take_value::<i32>().await.unwrap(), 42);
        assert!(context.get_value::<i32>().await.is_err());
    }

    #[tokio::test]
    async fn test_take_lazy_value() {
        let context = Context::new();

        context.provide_lazy(|| async { Ok::<i32, ProvideError>(42) });
        assert_eq!(context.take_value::<i32>().await.unwrap(), 42);
        assert!(context.get_value::<i32>().await.is_err());
    }
}
