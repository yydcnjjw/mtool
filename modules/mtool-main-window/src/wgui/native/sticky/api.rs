use std::any::type_name;

use crate::wgui::generic::view::sticky;

use super::window::StickyWindow;
use mapp::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use tauri::Emitter;
use yew::prelude::*;

pub async fn show_sub_view<View, Id, Data>(
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
        sticky::Command::ShowSubview(sticky::TemplateView {
            id: id.to_string(),
            template_id: type_name::<View>().to_string(),
            template_data: serde_json::to_value(data)?,
        }),
    )?;
    Ok(())
}
