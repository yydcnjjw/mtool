#![feature(once_cell)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::thread;

use cmder_mod::{ClosureCmd, ServiceClient as CmderCli};
use keybinding_mod::ServiceClient as KeybindingCli;

use tauri::Manager;

mod dict;

fn window_toggle<R>(win: tauri::Window<R>) -> tauri::Result<()>
where
    R: tauri::Runtime,
{
    if win.is_visible()? {
        win.hide()?;
    } else {
        win.show()?;
        win.set_focus()?;
    }
    Ok(())
}

pub async fn load(keybindingcli: KeybindingCli, cmder: CmderCli) -> anyhow::Result<()> {
    cmder
        .add(
            "test".into(),
            ClosureCmd::new(move |_| {
                println!("test");
            }),
        )
        .await?;
    keybindingcli
        .define_key_binding("C-A-l".into(), "test".into())
        .await??;

    let async_rt = tokio::runtime::Handle::current();

    thread::spawn(move || {
        let _guard = async_rt.enter();
        tauri::async_runtime::set(async_rt);

        #[cfg(target_os = "windows")]
        unsafe {
            windows::Win32::UI::Controls::InitCommonControls();
        }

        let context = tauri::generate_context!();
        tauri::Builder::default()
            .any_thread()
            .setup(move |app| {
                let win = app.get_window("main").unwrap();
                win.center().unwrap();
                let mut pos = win.outer_position().unwrap();
                pos.y = 100;
                win.set_position(pos).unwrap();

                let app = app.app_handle();
                tokio::spawn(async move {
                    if let Err(e) = {
                        cmder
                            .add(
                                "anynav_window_toggle".into(),
                                ClosureCmd::new(move |_| {
                                    let win = app.get_window("main").unwrap();
                                    if let Err(e) = window_toggle(win) {
                                        log::error!("{:?}", e);
                                    }
                                }),
                            )
                            .await?;
                        keybindingcli
                            .define_key_binding(
                                "C-A-<Spacebar>".into(),
                                "anynav_window_toggle".into(),
                            )
                            .await??;
                        Ok::<(), anyhow::Error>(())
                    } {
                        log::error!("{:?}", e);
                    }
                    Ok::<(), anyhow::Error>(())
                });

                Ok(())
            })
            .plugin(dict::init())
            .run(context)
            .expect("Failed to running anynav");
    });

    Ok(())
}
