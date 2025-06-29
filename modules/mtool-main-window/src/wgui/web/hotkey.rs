use std::{cell::RefCell, rc::Rc};

use mapp::{
    anyhow,
    prelude::*,
    serde::Serialize,
    tracing::{debug, warn},
};
use mkeybinding::KeyMap;
use mtool_cmder::LocalCmder;
use mtool_wgui::{mtauri_sys::prelude::*, Keybinding, SharedAction};
use yew::platform::spawn_local;

use crate::wgui::generic::hotkey::{Hotkey, HotkeyMap};

pub async fn init(
    keybinding: Res<Keybinding>,
    cmder: Res<LocalCmder>,
    injector: LocalInjector,
) -> Result<(), anyhow::Error> {
    async fn exec_command_when(
        cmder: Res<LocalCmder>,
        injector: LocalInjector,
        command: String,
        _when: Option<String>,
    ) -> Result<(), anyhow::Error> {
        debug!("{}", command);

        if let Some(cmd) = cmder.get_command_with_name(&command) {
            cmd.exec_local(&injector).await?;
        } else {
            #[derive(Serialize)]
            #[serde(crate = "mapp::serde")]
            struct Args {
                command: String,
            }

            spawn_local(async move {
                if let Err(e) =
                    invoke::<_, ()>("plugin:mtool-main-window|exec_command", &Args { command })
                        .await
                {
                    warn!("{:}", e);
                }
            });
        }

        Ok(())
    }

    async fn register_keybinding_inner(
        keybinding: Res<Keybinding>,
        cmder: Res<LocalCmder>,
        injector: LocalInjector,
    ) -> Result<(), anyhow::Error> {
        match invoke::<(), HotkeyMap>("plugin:mtool-main-window|window_hotkeys", &()).await {
            Ok(kbs) => {
                let km = KeyMap::<SharedAction>::new_with_vec(
                    kbs.0
                        .into_iter()
                        .map(|Hotkey { command, kbd, when }| {
                            let cmder = cmder.clone();
                            let injector = injector.clone();
                            debug!(
                                "register keybinding: {} -> {} when {:?}",
                                command, kbd, when
                            );
                            (
                                kbd,
                                Rc::new(RefCell::new(move || {
                                    exec_command_when(
                                        cmder.clone(),
                                        injector.clone(),
                                        command.clone(),
                                        when.clone(),
                                    )
                                })) as SharedAction,
                            )
                        })
                        .collect::<Vec<_>>(),
                )?;
                keybinding.push_keymap("global", km);
            }
            Err(e) => warn!("{:?}", e),
        }
        Ok(())
    }

    spawn_local(async move {
        if let Err(e) = register_keybinding_inner(keybinding, cmder, injector).await {
            warn!("{:?}", e)
        }
    });
    Ok(())
}
