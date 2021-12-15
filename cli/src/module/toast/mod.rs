use anyhow::Context;
use async_trait::async_trait;
use clap::Parser;
use notify_rust::{Notification, Timeout};

use crate::{
    app::App,
    core::command::{self, AddCommand, Command},
};

struct Toast {}

/// Toast module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Executable name
    #[clap(long)]
    appname: Option<String>,

    /// Single line to summarize the content
    #[clap(long)]
    summary: Option<String>,

    /// Subtitle for macOS
    #[clap(long)]
    subtitle: Option<String>,

    /// Multiple lines possible
    #[clap(long)]
    body: Option<String>,

    /// Use a file:// URI or a name in an icon theme, must be compliant freedesktop.org
    #[clap(long)]
    icon: Option<String>,

    /// Lifetime of the Notification in ms
    #[clap(long)]
    timeout: Option<u32>,
}

#[async_trait]
impl Command for Toast {
    async fn exec(&mut self, args: Vec<String>) -> anyhow::Result<command::Output> {
        let args = Args::try_parse_from(args).context("Toast parse args failed")?;

        let mut notify = Notification::new();
        if let Some(appname) = args.appname.as_ref() {
            notify.appname(appname);
        }

        if let Some(summary) = args.summary.as_ref() {
            notify.summary(summary);
        }

        if let Some(subtitle) = args.subtitle.as_ref() {
            notify.subtitle(subtitle);
        }

        if let Some(body) = args.body.as_ref() {
            notify.body(body);
        }

        if let Some(icon) = args.icon.as_ref() {
            notify.icon(icon);
        }

        notify.timeout(match args.timeout {
            Some(n) => Timeout::Milliseconds(n),
            None => Timeout::Never,
        });

        notify.show().context("Show notify failed")?;
        Ok(command::Output::None)
    }
}

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let sender = &app.evbus.sender();
    AddCommand::post(sender, "toast".into(), Toast {}).await?;
    Ok(())
}
