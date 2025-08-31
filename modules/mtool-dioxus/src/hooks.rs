use std::{
    any::type_name,
    future::Future,
    sync::atomic::{AtomicUsize, Ordering},
};

use dioxus::prelude::*;
use mapp::{anyhow, prelude::*};

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

// pub fn use_app_context_provider<T, F, O>(f: F) -> Resource<T>
// where
//     T: Send + Sync + Clone + 'static,
//     F: FnMut() -> O + 'static,
//     O: Future<Output = Result<T, anyhow::Error>> + 'static,
// {
//     let injector: Injector = use_context();

//     use_resource(move || {
//         to_owned![injector];
//         async move {
//             match injector.get_without_construct::<T>().await {
//                 Some(value) => value,
//                 None => {
//                     let value = f().await.unwrap();
//                     injector.insert(value.clone());
//                     value
//                 }
//             }
//         }
//     })
// }
