use anyhow::Context;
use clap::Parser;
use mapp::provider::{Res, Take};
use mtool_cmder::CommandArgs;
use mtool_interactive::OutputDevice;
use notify_rust::{Notification, Timeout};

/// Toast module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(no_binary_name = true)]
struct Args {
    /// Executable name
    #[clap(long)]
    appname: Option<String>,

    /// Single line to summarize the content
    #[clap(long)]
    summary: String,

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
    #[clap(long, default_value_t = 5000)]
    timeout: u32,
}

pub async fn toast(args: Take<CommandArgs>, o: Res<OutputDevice>) -> Result<(), anyhow::Error> {
    let args = match Args::try_parse_from(args.take()?.iter()) {
        Ok(args) => args,
        Err(e) => {
            o.output(&e.render().to_string()).await?;
            return Ok(());
        }
    };

    let mut notify = Notification::new();
    if let Some(appname) = args.appname.as_ref() {
        notify.appname(appname);
    }

    notify.summary(&args.summary);

    if let Some(subtitle) = args.subtitle.as_ref() {
        notify.subtitle(subtitle);
    }

    if let Some(body) = args.body.as_ref() {
        notify.body(body);
    }

    if let Some(icon) = args.icon.as_ref() {
        notify.icon(icon);
    }

    notify.timeout(Timeout::Milliseconds(args.timeout));

    notify.show().context("Failed to show notify")?;

    Ok(())
}
