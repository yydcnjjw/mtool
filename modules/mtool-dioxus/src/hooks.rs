use std::{
    any::type_name,
    sync::atomic::{AtomicUsize, Ordering},
};

use dioxus::prelude::*;
use mapp::prelude::*;

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

pub fn use_app_resource<T>() -> Resource<T>
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
