use std::any::type_name;

use crate::wgui::generic::{view::sticky, STICKY_WINDOW_LABEL};

use super::window::StickyWindow;
use mapp::prelude::*;
use mtool_wgui::component::{ProgressNotification, ProgressNotificationProps};
use serde::{de::DeserializeOwned, Serialize};
use tauri::Emitter;
use yew::prelude::*;

pub async fn show_main<View, Id, Data>(
    win: Res<StickyWindow>,
    id: Id,
    data: Data,
) -> Result<(), anyhow::Error>
where
    View: BaseComponent + 'static,
    View::Properties: DeserializeOwned,
    Id: ToString,
    Data: Serialize,
{
    win.emit_to(
        win.label(),
        "sticky:command",
        sticky::Command::ShowMain(sticky::TemplateView {
            id: id.to_string(),
            template_id: type_name::<View>().to_string(),
            template_data: serde_json::to_value(data)?,
        }),
    )?;
    Ok(())
}

pub async fn show_sub<View, Id, Data, Emitter, R>(
    emitter: &Emitter,
    id: Id,
    data: Data,
) -> Result<(), anyhow::Error>
where
    Emitter: tauri::Emitter<R>,
    R: tauri::Runtime,
    View: BaseComponent + 'static,
    View::Properties: DeserializeOwned,
    Id: ToString,
    Data: Serialize,
{
    emitter.emit_to(
        STICKY_WINDOW_LABEL,
        "sticky:command",
        sticky::Command::ShowSub(sticky::TemplateView {
            id: id.to_string(),
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
    show_sub::<ProgressNotification, _, _, _, _>(emitter, "sub", props).await
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
