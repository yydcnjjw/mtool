use std::any::type_name;

use crate::wgui::generic::{view::sticky, STICKY_WINDOW_LABEL};

use mapp::{
    anyhow,
    serde::{de::DeserializeOwned, Serialize},
    serde_json,
};
use mtool_wgui::component::{ProgressNotification, ProgressNotificationProps};
use yew::prelude::*;

pub async fn show_main<View, Emitter, R>(
    emitter: &Emitter,
    data: View::Properties,
) -> Result<(), anyhow::Error>
where
    View: BaseComponent + 'static,
    View::Properties: Serialize + DeserializeOwned,
    Emitter: tauri::Emitter<R>,
    R: tauri::Runtime,
{
    emitter.emit_to(
        STICKY_WINDOW_LABEL,
        "sticky:command",
        sticky::Command::ShowMain(sticky::TemplateView {
            id: "".into(),
            template_id: type_name::<View>().to_string(),
            template_data: serde_json::to_value(data)?,
        }),
    )?;
    Ok(())
}

pub async fn show_progress_notification<Emitter, R>(
    emitter: &Emitter,
    props: ProgressNotificationProps,
) -> Result<(), anyhow::Error>
where
    Emitter: tauri::Emitter<R>,
    R: tauri::Runtime,
{
    show_main::<ProgressNotification, _, _>(emitter, props).await
}

pub fn left_mouse_up<Emitter, R>(emitter: &Emitter) -> Result<(), anyhow::Error>
where
    Emitter: tauri::Emitter<R>,
    R: tauri::Runtime,
{
    emitter.emit_to(
        STICKY_WINDOW_LABEL,
        "sticky:command",
        sticky::Command::LeftMouseUp,
    )?;
    Ok(())
}
