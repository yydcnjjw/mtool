use chrono::Timelike;
use dioxus::prelude::*;
use mapp::{
    anyhow,
    futures::future,
    prelude::*,
    serde::{Deserialize, Serialize},
};
use mtool_cmdpal::{Command, CommandItem, CommandPalette, CommandResult};
use mtool_core::ConfigStore;
use mtool_storage::crdt::{self, CrdtService};

use crate::Config;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum AssistantMode {
    Desktop,
    RemoteDesktop,
}

pub struct AssistantContext {
    mode: crdt::State<AssistantMode>,
    pub config: Config,
}

impl AssistantContext {
    pub fn mode(&self) -> AssistantMode {
        *self.mode.borrow()
    }

    pub fn set_mode(&self, mode: AssistantMode) {
        self.mode.set(mode);
    }

    pub fn toggle_mode(&self) -> AssistantMode {
        let mode = match self.mode() {
            AssistantMode::Desktop => AssistantMode::RemoteDesktop,
            AssistantMode::RemoteDesktop => AssistantMode::Desktop,
        };

        self.set_mode(mode);
        mode
    }

    pub fn is_desktop(&self) -> bool {
        cfg!(feature = "desktop")
    }

    pub fn is_mobile(&self) -> bool {
        cfg!(feature = "mobile")
    }

    pub fn request_address(&self) -> Option<String> {
        if self.is_desktop() {
            self.config.mobile_rpc_address.clone()
        } else {
            self.config.desktop_rpc_address.clone()
        }
    }
}

impl AssistantContext {
    pub async fn construct(
        cs: Res<ConfigStore>,
        crdt: Res<CrdtService>,
    ) -> Result<Res<Self>, anyhow::Error> {
        let cfg = cs.get_optional::<Config>("assistant").unwrap_or_default();

        let mode = crdt::State::new_with(crdt, "assistant.notify_mode", move || async move {
            let hour = chrono::Local::now().hour();
            Ok(if hour < 18 && hour > 9 {
                AssistantMode::RemoteDesktop
            } else {
                AssistantMode::Desktop
            })
        })
        .await?;

        Ok(Res::new(AssistantContext { mode, config: cfg }))
    }

    pub fn mode_change_signal(&self) -> ReadOnlySignal<AssistantMode> {
        let mut rx = self.mode.subscribe();

        let mut signal = use_signal(|| *rx.borrow());

        use_hook(|| {
            spawn(async move {
                while let Ok(_) = rx.changed().await {
                    signal.set(*rx.borrow_and_update());
                }
            })
        });
        signal.into()
    }
}

pub(crate) async fn register_commands(
    cmdpal: Res<CommandPalette>,
    ctx: Res<AssistantContext>,
) -> Result<(), anyhow::Error> {
    cmdpal
        .add_top_level_command(CommandItem::from(
            Command::new("Remote desktop mode", {
                to_owned![ctx];
                move || {
                    ctx.set_mode(AssistantMode::RemoteDesktop);
                    future::ok(CommandResult::Dismiss)
                }
            })
            .description("set remote desktop mode"),
        ))
        .add_top_level_command(CommandItem::from(
            Command::new("Desktop mode", {
                to_owned![ctx];
                move || {
                    ctx.set_mode(AssistantMode::Desktop);
                    future::ok(CommandResult::Dismiss)
                }
            })
            .description("set desktop mode"),
        ));

    Ok(())
}
