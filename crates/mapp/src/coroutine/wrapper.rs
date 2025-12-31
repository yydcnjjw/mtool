use std::marker::PhantomData;

pub struct FuncWrapper<Func, T> {
    pub func: Func,
    _marker: PhantomData<T>,
}

impl<Func, T> FuncWrapper<Func, T> {
    pub fn new(func: Func) -> Self {
        Self {
            func,
            _marker: PhantomData,
        }
    }
}
