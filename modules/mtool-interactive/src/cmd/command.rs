use mapp::prelude::*;
use mtool_cmder::SharedCommandPtr;
use mtool_wgui::Templator;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::*;

#[derive(Properties, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CommandItem {
    name: String,
    descrption: String,
    #[serde(skip)]
    cmd: Option<SharedCommandPtr>,
}

impl PartialOrd for CommandItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for CommandItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl From<SharedCommandPtr> for CommandItem {
    fn from(value: SharedCommandPtr) -> Self {
        Self {
            name: value.get_name().into(),
            descrption: value.get_descrption().into(),
            cmd: Some(value),
        }
    }
}

impl CompleteItem for CommandItem {
    type WGuiView = CommandItemView;

    fn complete_hint(&self) -> String {
        self.name.clone()
    }
}

#[function_component]
pub fn CommandItemView(props: &CommandItem) -> Html {
    html! {
        <div class={classes!(
            "flex",
            "items-center",
            "h-10",
        )}>
          <span class={classes!("align-middle")} title={ props.name.clone() }>{ props.descrption.clone() }</span>
        </div>
    }
}

pub async fn web_init(templator: Res<Templator>) -> Result<(), anyhow::Error> {
    templator.add_template::<CommandItemView>();
    Ok(())
}

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        use mtool_system::keybinding::Keybinding;
    }
}

#[cfg(not(target_family = "wasm"))]
pub async fn init(keybinding: Res<Keybinding>) -> Result<(), anyhow::Error> {
    use anyhow::Context;
    use itertools::Itertools;
    use mtool_cmder::{Cmder, CommandArgs};

    pub async fn exec_command(
        c: Res<Completion>,
        cmder: Res<Cmder>,
        injector: Injector,
    ) -> Result<(), anyhow::Error> {
        let command = {
            let cmder = cmder.clone();
            let command = c
                .complete_read(
                    CompletionArgs::with_vec(
                        cmder
                            .iter()
                            .map(|v| CommandItem::from(v.clone()))
                            .sorted()
                            .collect_vec(),
                    )
                    .prompt("Input command..."),
                )
                .await?;
            match command {
                Some(command) => command,
                None => return Ok(()),
            }
        };

        {
            let command = command.clone();
            injector.construct_once(move || async move {
                let completed = c
                    .complete_read(
                        CompletionArgs::<String>::without_completion().prompt(&command.name),
                    )
                    .await?
                    .context("complete read canceled")?;
                Ok(Take::new(CommandArgs::new(shellwords::split(&completed)?)))
            });
        }

        command.cmd.unwrap().exec(&injector).await?;
        Ok(())
    }

    keybinding.define_global("M-A-x", exec_command).await?;
    Ok(())
}
