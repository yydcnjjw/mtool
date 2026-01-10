use std::fmt;

pub struct Take<T>(T);

impl<T> Take<T> {
    pub fn new(val: T) -> Self {
        Self(val)
    }
}

impl<T> Clone for Take<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> fmt::Debug for Take<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Take").field(&self.0).finish()
    }
}

pub type TakeOpt<T> = Option<Take<T>>;

