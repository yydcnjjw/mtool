use mapp::prelude::*;
use mtool_cmder::Cmder;
use mtool_core::ConfigStore;
use mtool_wgui::Builder;
use tauri::{command, generate_handler, State};

use crate::wgui::generic::hotkey::HotkeyMap;

use super::{main, sticky};

#[command]
pub async fn window_hotkeys(
    window: tauri::WebviewWindow,
    injector: State<'_, Injector>,
) -> Result<HotkeyMap, serde_error::Error> {
    Ok(match injector.get::<Res<ConfigStore>>().await {
        Ok(cs) => cs
            .get::<HotkeyMap>(&format!("wgui.{}.hotkey", window.label()))
            .await
            .unwrap_or_default(),
        _ => HotkeyMap::default(),
    })
}

pub(crate) async fn setup(
    builder: Res<Builder>,
    injector: Injector,
    cmder: Res<Cmder>,
) -> Result<(), anyhow::Error> {
    builder
        // .setup_with_app(|app|{
        //     app.wry_plugin(sticky::WryPluginBuilder::new());
        //     Ok(())
        // })
        .setup(|builder| {
        Ok(builder.plugin(
            tauri::plugin::Builder::<_, ()>::new("mtool-main-window")
                .setup(move |app, _| {
                    main::plugin_setup(app, cmder, injector.construct_oneshot())?;
                    Ok(())
                })
                .invoke_handler(generate_handler![window_hotkeys, main::exec_command])
                .build(),
        ))
    })?;
    Ok(())
}
