use dioxus::prelude::*;
use mapp::{anyhow, prelude::*};
use mtool_cmdpal::{Command, CommandItem, CommandPalette, CommandResult};
use mtool_dioxus::prelude::*;

use crate::{platforms::Searcher, view, AppsSource, WindowsSource};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-apps");
    group.add_module(Module);
    group
}

struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(DioxusStage::Setup, dioxus_setup);
        Ok(())
    }
}

async fn apps() -> Result<CommandResult, anyhow::Error> {
    Ok(CommandResult::ShowView(|| rsx! { view::Apps {  } }))
}

async fn dioxus_setup(
    cmdpal: Res<CommandPalette>,
    builder: Res<DioxusBuilder>,
) -> Result<(), anyhow::Error> {
    let searcher = Searcher::new(vec![])?;

    let app_source = AppsSource::new(searcher.watch_apps());

    builder.with_launch_builder({
        to_owned![app_source];
        move |builder| builder.with_context(app_source)
    });

    cmdpal
        .add_top_level_command(CommandItem::from(
            Command::new("Apps", apps).description("List apps"),
        ))
        .add_source(app_source)
        .add_source(WindowsSource::new(searcher.watch_windows()));
    Ok(())
}
