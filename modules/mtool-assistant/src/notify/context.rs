use chrono::Timelike;
use dioxus::prelude::*;
use mapp::{
    anyhow::{self, Context},
    futures::future,
    prelude::*,
    rand::{seq::SliceRandom, thread_rng},
    serde::{Deserialize, Serialize},
    serde_json,
    sync::{lock_api::MutexGuard, Mutex, RawMutex},
    tokio::{self},
    tracing::{debug, warn},
};
use mtool_cmdpal::{Command, CommandItem, CommandPalette, CommandResult};
use mtool_core::ConfigStore;
use mtool_storage::crdt::{self, CrdtService};
use mtool_system::{AppInfo, MediaNotification, Notification, SystemEventSource, SystenEvent};
use std::{
    borrow::Borrow,
    io::{BufReader, Cursor},
};

use crate::{rpc, Config};

use super::media;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum NotifyMode {
    Desktop,
    RemoteDesktop,
}

static IS_DESKTOP: bool = cfg!(feature = "desktop");
static IS_MOBILE: bool = cfg!(feature = "mobile");

struct NotifyContextInner {
    is_playing: bool,
    notify_mode: crdt::State<NotifyMode>,
    config: Config,
}

pub struct NotifyContext {
    inner: Mutex<NotifyContextInner>,
}

impl NotifyContextInner {
    fn notify_mode(&self) -> NotifyMode {
        *self.notify_mode.borrow()
    }

    fn set_notify_mode(&self, mode: NotifyMode) {
        self.notify_mode.set(mode);
    }
}

impl NotifyContext {
    pub async fn construct(
        cs: Res<ConfigStore>,
        crdt: Res<CrdtService>,
    ) -> Result<Res<Self>, anyhow::Error> {
        let cfg = cs.get_optional::<Config>("assistant").unwrap_or_default();

        let notify_mode =
            crdt::State::new_with(crdt, "assistant.notify_mode", move || async move {
                let hour = chrono::Local::now().hour();
                Ok(if hour < 18 && hour > 9 {
                    NotifyMode::RemoteDesktop
                } else {
                    NotifyMode::Desktop
                })
            })
            .await?;

        Ok(Res::new(NotifyContext {
            inner: Mutex::new(NotifyContextInner {
                is_playing: false,
                notify_mode,
                config: cfg,
            }),
        }))
    }
}

impl NotifyContext {
    fn lock(&self) -> MutexGuard<'_, RawMutex, NotifyContextInner> {
        self.inner.lock()
    }

    async fn post_notification(
        ctx: Res<Self>,
        notification: Notification,
    ) -> Result<(), anyhow::Error> {
        let cfg = ctx.lock().config.clone();

        let post_addr = if IS_DESKTOP {
            cfg.mobile_rpc_address
        } else {
            cfg.desktop_rpc_address
        };

        if let Some(addr) = post_addr {
            let mut client = rpc::MtoolAssistantClient::connect(addr).await?;

            let request = tonic::Request::new(rpc::Notification {
                data: serde_json::to_vec(&notification)?,
            });

            _ = client.post_notification(request).await?;
        }
        Ok(())
    }

    async fn handle_im_notification(ctx: Res<Self>, app: AppInfo) -> Result<(), anyhow::Error> {
        let (notify_mode, cfg) = {
            let ctx = ctx.lock();
            (ctx.notify_mode(), ctx.config.clone())
        };

        let do_play = match notify_mode {
            NotifyMode::Desktop => IS_DESKTOP,
            NotifyMode::RemoteDesktop => !IS_DESKTOP,
        };

        if do_play {
            Self::play(ctx)
        } else {
            Self::post_notification(ctx, Notification::Im { app }).await
        }
    }

    async fn handle_media_notification(
        ctx: Res<Self>,
        media: MediaNotification,
    ) -> Result<(), anyhow::Error> {
        if IS_MOBILE {
            return Self::post_notification(ctx, Notification::Media(media)).await;
        }

        let lyric = media::get_netease_lyrics(media.metadata.id).await;

        Ok(())
    }

    pub async fn handle_notification_posted(
        ctx: Res<Self>,
        notification: Notification,
    ) -> Result<(), anyhow::Error> {
        match notification {
            Notification::Im { app } => Self::handle_im_notification(ctx, app).await,
            Notification::Media(media) => Self::handle_media_notification(ctx, media).await,
            _ => Ok(()),
        }
    }

    pub async fn init(
        ctx: Res<Self>,
        source: Res<SystemEventSource>,
        #[cfg(feature = "desktop")] cmdpal: Res<CommandPalette>,
    ) -> Result<(), anyhow::Error> {
        {
            to_owned![ctx];
            tokio::spawn(async move {
                let mut rx = source.subscribe();
                while let Ok(ev) = rx.recv().await {
                    to_owned![ctx];

                    debug!("{:?}", ev);
                    tokio::spawn(async move {
                        if let Err(e) = match ev {
                            SystenEvent::NotificationPosted(notification) => {
                                Self::handle_notification_posted(ctx.clone(), notification).await
                            }
                        } {
                            warn!("{:?}", e);
                        }
                    });
                }
            });
        }

        #[cfg(feature = "desktop")]
        {
            cmdpal
                .add_top_level_command(CommandItem::from(
                    Command::new("Remote desktop mode", {
                        to_owned![ctx];
                        move || {
                            ctx.set_notify_mode(NotifyMode::RemoteDesktop);
                            future::ok(CommandResult::Dismiss)
                        }
                    })
                    .description("set remote desktop mode"),
                ))
                .add_top_level_command(CommandItem::from(
                    Command::new("Desktop mode", {
                        to_owned![ctx];
                        move || {
                            ctx.set_notify_mode(NotifyMode::Desktop);
                            future::ok(CommandResult::Dismiss)
                        }
                    })
                    .description("set desktop mode"),
                ));
        }

        Ok(())
    }

    pub fn set_notify_mode(&self, mode: NotifyMode) {
        self.lock().set_notify_mode(mode);
    }

    pub async fn toggle_notify_mode_with_sync(ctx: Res<Self>) -> Result<NotifyMode, anyhow::Error> {
        let mode = match ctx.lock().notify_mode() {
            NotifyMode::Desktop => NotifyMode::RemoteDesktop,
            NotifyMode::RemoteDesktop => NotifyMode::Desktop,
        };

        ctx.set_notify_mode(mode);
        Ok(mode)
    }

    pub fn new_notify_mode_signal(&self) -> ReadOnlySignal<NotifyMode> {
        let mut rx = self.lock().notify_mode.subscribe();

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

    fn play(ctx: Res<Self>) -> Result<(), anyhow::Error> {
        tokio::task::spawn_blocking(move || {
            {
                let mut ctx = ctx.lock();

                if ctx.is_playing {
                    return Ok(());
                }

                ctx.is_playing = true;
            }

            let (_stream, handle) = rodio::OutputStream::try_default()?;

            let sink = rodio::Sink::try_new(&handle)?;

            let notify_assets = asset!("/assets/voices/notify");

            let voices = include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets/voices/notify");

            let voice = voices
                .entries()
                .choose(&mut thread_rng())
                .context("notify voice list is empty")?;
            let resp = dioxus_asset_resolver::serve_asset(
                &notify_assets
                    .resolve()
                    .join(voice.path())
                    .display()
                    .to_string(),
            )?;
            sink.append(rodio::Decoder::new(BufReader::new(Cursor::new(
                resp.into_body(),
            )))?);

            sink.sleep_until_end();

            ctx.lock().is_playing = false;

            Ok::<_, anyhow::Error>(())
        });
        Ok(())
    }
}
