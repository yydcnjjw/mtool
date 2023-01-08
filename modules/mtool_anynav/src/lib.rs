// #![feature(once_cell)]
// #![cfg_attr(
//     all(not(debug_assertions), target_os = "windows"),
//     windows_subsystem = "windows"
// )]

// use std::thread;

// use anyhow::Context;
// use cmder_mod::{ClosureCmd, ServiceClient as CmderCli};
// use config_mod::ServiceClient as ConfigCli;
// use keybinding_mod::ServiceClient as KeybindingCli;

// use serde::{Deserialize, Serialize};
// use tauri::{AppHandle, Manager, GlobalShortcutManager};

// mod dict;

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Config {
//     dict: dict::Config,
// }

// fn window_toggle<R>(win: tauri::Window<R>) -> tauri::Result<()>
// where
//     R: tauri::Runtime,
// {
//     if win.is_visible()? {
//         win.hide()?;
//     } else {
//         win.show()?;
//         win.set_focus()?;
//     }
//     Ok(())
// }

// async fn register_window_toogle_shortcut(
//     app: AppHandle,
//     keybindingcli: KeybindingCli,
//     cmder: CmderCli,
// ) -> anyhow::Result<()> {
//     {
//         let app = app.clone();
//         let cmder = cmder.clone();
//         cmder
//             .add(
//                 "anynav_window_toggle".into(),
//                 ClosureCmd::new(move |_| {
//                     let win = app.get_window("main").unwrap();
//                     if let Err(e) = window_toggle(win) {
//                         log::error!("{:?}", e);
//                     }
//                 }),
//             )
//             .await?;
//     }

//     if cfg!(windows) {
//         app.global_shortcut_manager().register("Ctrl+Alt+Space", move || {
//             let cmder = cmder.clone();
//             tokio::spawn(async move {
//                 if let Err(e) = cmder.exec("anynav_window_toggle".into(), Vec::new()).await {
//                     log::error!("{:?}", e);
//                 }
//             });
            
//         }).context("Failed to register shortcut C-A-<Spacebar>")?;
//     } else {
//         // BUG: window event can not captured at tauri
//         // maybe webview2 capture all event
//         // https://github.com/MicrosoftEdge/WebView2Feedback/issues/468
//         keybindingcli
//             .define_key_binding("C-A-<Spacebar>".into(), "anynav_window_toggle".into())
//             .await??;
//     }
//     Ok(())
// }

// pub async fn load(
//     keybindingcli: KeybindingCli,
//     cmder: CmderCli,
//     cfgcli: ConfigCli,
// ) -> anyhow::Result<()> {
//     let async_rt = tokio::runtime::Handle::current();

//     let cfg: Config = cfgcli.get_value("anynav".into()).await??.try_into()?;

//     let dict_plug = dict::init(cfg.dict).await?;

//     thread::spawn(move || {
//         let _guard = async_rt.enter();
//         tauri::async_runtime::set(async_rt);

//         let context = tauri::generate_context!();
//         tauri::Builder::default()
//             .any_thread()
//             .setup(move |app| {
//                 let win = app.get_window("main").unwrap();
//                 win.center().unwrap();
//                 let mut pos = win.outer_position().unwrap();
//                 pos.y = 100;
//                 win.set_position(pos).unwrap();

//                 let app = app.app_handle();
//                 tokio::spawn(async move {
//                     if let Err(e) = register_window_toogle_shortcut(app, keybindingcli, cmder).await {
//                         log::error!("{:?}", e);
//                     }
//                 });

//                 Ok(())
//             })
//             .plugin(dict_plug)
//             .run(context)
//             .expect("Failed to running anynav");
//     });

//     Ok(())
// }


use async_trait::async_trait;
use mapp::{AppContext, AppModule};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, _app: &mut AppContext) -> Result<(), anyhow::Error> {

        Ok(())
    }
}
