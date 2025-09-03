use std::time::Duration;

use dioxus_primitives::toast::{ToastOptions, Toasts};
use mapp::anyhow;

pub fn toast_err<E>(toast: Toasts, e: E)
where
    E: Into<anyhow::Error>,
{
    toast.error(
        "Error occurred".into(),
        ToastOptions::default()
            .description(e.into())
            .duration(Duration::from_secs(3)),
    );
}
