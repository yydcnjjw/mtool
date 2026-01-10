use futures::{FutureExt, future::LocalBoxFuture};
use minject::LocalProvide;
use std::{any::type_name, cell::RefCell, rc::Rc, sync::Arc};

use crate::inject::{LocalContainer, Take};

use super::Error;

pub struct Context {
    local_injector: RefCell<LocalContainer>,
}

impl Context {
    
    
    
    
}

impl<T> LocalProvide<Rc<T>> for Context
where
    T: 'static,
{
    type Error = Error;

    fn local_provide(&self) -> LocalBoxFuture<'_, Result<Rc<T>, Self::Error>> {
        async {
            self.local_injector
                .borrow()
                .consume::<Rc<T>>()
                .await
                .ok_or_else(|| Error::ProvideNotFound {
                    value: type_name::<T>(),
                })
        }
        .boxed_local()
    }
}

impl<T> LocalProvide<Arc<T>> for Context
where
    T: 'static,
{
    type Error = Error;

    fn local_provide(&self) -> LocalBoxFuture<'_, Result<Arc<T>, Self::Error>> {
        async {
            self.local_injector
                .borrow()
                .consume::<Arc<T>>()
                .await
                .ok_or_else(|| Error::ProvideNotFound {
                    value: type_name::<T>(),
                })
        }
        .boxed_local()
    }
}

impl<T> LocalProvide<Take<T>> for Context
where
    T: 'static,
{
    type Error = Error;

    fn local_provide(&self) -> LocalBoxFuture<'_, Result<Take<T>, Self::Error>> {
        async {
            self.local_injector
                .borrow_mut()
                .remove::<T>()
                .map(|value| Take::new(value))
                .ok_or_else(|| Error::ProvideNotFound {
                    value: type_name::<T>(),
                })
        }
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
