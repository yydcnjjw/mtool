use crate::component::ProgressNotificationProps;
use tauri::{Emitter, Runtime};

pub fn update_progress_notification<T, R>(
    emitter: &T,
    props: ProgressNotificationProps,
) -> Result<(), tauri::Error>
where
    T: Emitter<R>,
    R: Runtime,
{
    Ok(emitter.emit("update_progress_notification", props)?)
}
