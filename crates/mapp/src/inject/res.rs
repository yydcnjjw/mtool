use std::{any::Any, fmt, ops::Deref, rc::Rc};

pub struct Res<T: ?Sized>(Rc<T>);

impl<T> Res<T> {
    pub fn new(val: T) -> Self {
        Self(Rc::new(val))
    }

    pub fn new_raw(val: Rc<T>) -> Self {
        Self(val)
    }

    pub fn try_unwrap(this: Self) -> Result<T, Rc<T>> {
        Rc::try_unwrap(this.0)
    }
}

impl Res<dyn Any> {
    pub fn downcast<T>(self) -> Result<Res<T>, Rc<dyn Any + 'static>>
    where
        T: 'static,
    {
        self.0.downcast().map(|value| Res(value))
    }
}

impl<T: ?Sized> Deref for Res<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> PartialEq for Res<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Clone for Res<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> fmt::Debug for Res<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Res").field(&self.0).finish()
    }
}
