use std::{cell::UnsafeCell, mem};

use futures::future::LocalBoxFuture;

enum State<T, F> {
    Uninit(F),
    Init(T),
    Poisoned,
}

pub struct LazyCell<T, F = fn() -> LocalBoxFuture<'static, T>> {
    state: UnsafeCell<State<T, F>>,
}

impl<T, F> LazyCell<T, F>
where
    F: FnOnce() -> LocalBoxFuture<'static, T>,
{
    pub const fn new(f: F) -> LazyCell<T, F> {
        LazyCell {
            state: UnsafeCell::new(State::Uninit(f)),
        }
    }

    pub async fn force(this: &LazyCell<T, F>) -> &T {
        let state = unsafe { &*this.state.get() };
        match state {
            State::Init(data) => data,
            State::Uninit(_) => unsafe { LazyCell::really_init(this).await },
            State::Poisoned => panic_poisoned(),
        }
    }

    #[cold]
    async unsafe fn really_init(this: &LazyCell<T, F>) -> &T {
        let state = unsafe { &mut *this.state.get() };

        let State::Uninit(f) = mem::replace(state, State::Poisoned) else {
            unreachable!()
        };

        let data = f().await;

        unsafe { this.state.get().write(State::Init(data)) };

        let state = unsafe { &*this.state.get() };
        let State::Init(data) = state else {
            unreachable!()
        };
        data
    }
}

#[cold]
#[inline(never)]
const fn panic_poisoned() -> ! {
    panic!("LazyCell instance has previously been poisoned")
}

#[cfg(test)]
mod tests {
    #[test]
    fn init() {}
}
