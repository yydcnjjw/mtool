use std::{
    any::type_name,
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};

use dioxus::prelude::*;
use mapp::{
    prelude::*,
    serde::{de::DeserializeOwned, Serialize},
};
use mtool_storage::crdt;

pub fn use_unique_id() -> Signal<String> {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    use_hook(move || {
        NEXT_ID.fetch_add(1, Ordering::Relaxed);
    });

    use_signal(|| {
        let id = NEXT_ID.load(Ordering::Relaxed);
        format!("mtool-dxc-{id}")
    })
}

pub fn use_app_context<T>() -> Resource<T>
where
    T: Send + Sync + Clone + 'static,
{
    let injector: Injector = use_context();

    use_resource(move || {
        to_owned![injector];
        async move {
            injector
                .get()
                .await
                .expect(&format!("Failed to get {}", type_name::<T>()))
        }
    })
}

pub async fn consume_app_context<T>() -> T
where
    T: Send + Sync + Clone + 'static,
{
    let injector: Injector = consume_context();
    injector
        .get()
        .await
        .expect(&format!("Failed to get {}", type_name::<T>()))
}

pub fn use_crdt_signal<T>(state: crdt::State<T>) -> ReadOnlySignal<T>
where
    T: Serialize + DeserializeOwned + fmt::Debug + Clone + Send + Sync + 'static,
{
    let (rx, mut signal) = use_hook(|| {
        let rx = state.subscribe();
        let value = rx.borrow().clone();
        (rx, Signal::new(value))
    });

    use_future(move || {
        to_owned![rx];
        async move {
            while let Ok(_) = rx.changed().await {
                signal.set(rx.borrow_and_update().clone());
            }
        }
    });

    signal.into()
}
