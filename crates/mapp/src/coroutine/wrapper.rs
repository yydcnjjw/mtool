use std::marker::PhantomData;

pub struct FuncWrapper<Func, Marker> {
    pub func: Func,
    _marker: PhantomData<Marker>,
}

impl<Func, Marker> FuncWrapper<Func, Marker> {
    pub fn new(func: Func) -> Self {
        Self {
            func,
            _marker: PhantomData,
        }
    }
}
