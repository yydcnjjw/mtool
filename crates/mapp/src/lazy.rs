use std::{cell::UnsafeCell, mem};
use std::future::Future;

use futures::future::LocalBoxFuture;

enum State<T, F> {
    Uninit(F),
    Init(T),
    Poisoned,
}

pub struct LazyCell<T, F = Box<dyn FnOnce() -> LocalBoxFuture<'static, T>>> {
    state: UnsafeCell<State<T, F>>,
}

impl<T> LazyCell<T> {
    pub fn new<F, Fut>(f: F) -> LazyCell<T>
    where
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = T> + 'static,
    {
        LazyCell {
            state: UnsafeCell::new(State::Uninit(Box::new(move || Box::pin(f())))),
        }
    }
}

impl<T, F, Fut> LazyCell<T, F>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T> + 'static,
{
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
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_lazy_init() {
        // Test scenario: Verify LazyCell can correctly initialize and
        // obtain value through force method.
        let cell = LazyCell::new(|| async { 42 });
        let value = LazyCell::force(&cell).await;
        assert_eq!(*value, 42);
    }

    #[tokio::test]
    async fn test_lazy_idempotent() {
        // Test scenario: Verify that multiple calls to the force
        // method return the same value, and the initialization logic
        // executes only once.
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        let cell = LazyCell::new(move || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                "hello"
            }
        });

        let val1 = LazyCell::force(&cell).await;
        assert_eq!(*val1, "hello");
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        let val2 = LazyCell::force(&cell).await;
        assert_eq!(*val2, "hello");
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
