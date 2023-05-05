use std::time::Duration;

use async_stream::stream;
use mtool_proxy_model::{Stats, TransferStats};
use mtool_wgui_core::{AutoResizeWindow, Horizontal, Vertical, WindowProps};
use tracing::{debug, warn};
use yew::{
    platform::{spawn_local, time},
    prelude::*,
};
use yew_icons::{Icon, IconId};

pub struct App {
    stats: Stats,
    diff_stats: Stats,
}

#[derive(Properties, PartialEq)]
pub struct AppProps {
    path: String,
}

pub enum AppMsg {
    UpdateStats(Stats),
}

impl Component for App {
    type Message = AppMsg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_stream(stream! {
            loop {
                time::sleep(Duration::from_secs(1)).await;
                match mtauri_sys::invoke::<(), Stats>("plugin:proxy|stats", &()).await {
                    Ok(stats) => yield AppMsg::UpdateStats(stats),
                    Err(e) => {
                        warn!("{:?}", e);
                        break;
                    }
                }
            }
        });

        Self {
            stats: Stats::default(),
            diff_stats: Stats::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::UpdateStats(stats) => self.update_stats(stats),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        spawn_local(async move {});

        html! {
            <>
            <AutoResizeWindow window={
                WindowProps{
                    horizontal: Horizontal::RightAlign,
                    vertical: Vertical::Absolute(24),
                    ..Default::default()
                }
            }>
              <div class={classes!(
                  "flex",
                  "flex-col",
                  "p-1",
                  "bg-gray-600/75",
              )}>
                {
                   for self.diff_stats.transfer.iter().map(|(k, v)| {
                     App::render_stats(k, v)
                   })
                }
              </div>
            </AutoResizeWindow>
            </>
        }
    }
}

impl App {
    fn render_stats(dest: &str, stats: &TransferStats) -> Html {
        html! {
            <div class={classes!(
                "flex",
                "justify-between",
                "text-white",
                "font-mono",
                "text-xs",
                "whitespace-nowrap",
            )}>
                <span class={classes!("text-left")}>{format!("{}:", dest)}</span>
                <div class={classes!(
                    "flex",
                    "justify-end",
                )}>
                  <Icon icon_id={IconId::OcticonsUpload16} width={"1em".to_owned()} height={"1em".to_owned()}/>
                  {Self::render_bandwidth(stats.tx)}
                  <Icon icon_id={IconId::OcticonsDownload16} width={"1em".to_owned()} height={"1em".to_owned()}/>
                  {Self::render_bandwidth(stats.rx)}
                </div>
            </div>
        }
    }

    fn render_bandwidth(n: usize) -> Html {
        const KB: usize = 1024;
        const KB_1: usize = KB - 1;
        const MB: usize = 1024 * KB;
        const MB_1: usize = MB - 1;
        const GB: usize = 1024 * MB;
        const GB_1: usize = GB - 1;

        let (n, unit) = match n {
            0..=KB_1 => (n as f32, "Bytes"),
            KB..=MB_1 => (n as f32 / KB as f32, "KB"),
            MB..=GB_1 => (n as f32 / MB as f32, "MB"),
            _ => (n as f32 / GB as f32, "GB"),
        };
        html! {
            <span class={classes!("w-[6rem]")}>{format!("{:.1}{}/s", n, unit)}</span>
        }
    }

    fn update_stats(&mut self, new: Stats) -> bool {
        self.diff_stats = App::diff_stats(new.clone(), &self.stats);
        self.stats = new;
        true
    }

    fn diff_stats(mut new: Stats, old: &Stats) -> Stats {
        debug!("{:?}, {:?}", new, old);

        for (dest, new_transfer) in &mut new.transfer {
            if let Some(old_transfer) = old.transfer.get(dest) {
                *new_transfer -= old_transfer.clone();
            }
        }
        new
    }
}
