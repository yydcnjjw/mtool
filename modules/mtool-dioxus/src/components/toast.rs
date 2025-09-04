use std::time::Duration;

use dioxus_primitives::toast::{ToastOptions, Toasts};
use mapp::{anyhow, tracing::warn};

pub fn toast_err<E>(toast: Toasts, e: E)
where
    E: Into<anyhow::Error>,
{
    let e = e.into();
    warn!("{e:?}");
    toast.error(
        "Error occurred".into(),
        ToastOptions::default()
            .description(e)
            .duration(Duration::from_secs(3)),
    );
}
