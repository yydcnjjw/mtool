use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;

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
