use chrono::Timelike;
use dioxus::prelude::*;
use mapp::{
    anyhow::{self, Context},
    prelude::*,
    rand::{seq::SliceRandom, thread_rng},
    sync::{lock_api::MutexGuard, Mutex, RawMutex},
    tokio,
    tracing::{debug, info, warn},
};
use mtool_cmdpal::{Command, CommandItem, CommandPalette, CommandResult};
use mtool_core::ConfigStore;
use mtool_system::{Notification, SystemEventSource, SystenEvent};
use std::{
    collections::HashSet,
    future::Future,
    io::{BufReader, Cursor},
};

use crate::{rpc, Config};

#[derive(Clone, Copy)]
pub enum NotifyMode {
    Desktop,
    RemoteDesktop,
}

static IS_DESKTOP: bool = cfg!(any(target_os = "windows", target_os = "linux"));

struct NotifyContextInner {
    pkg_white_list: HashSet<String>,
    is_playing: bool,
    notify_mode: NotifyMode,
    config: Config,
}

pub struct NotifyContext {
    inner: Mutex<NotifyContextInner>,
}

impl NotifyContext {
    pub async fn construct(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let cfg = cs.get_optional::<Config>("assistant").unwrap_or_default();

        let mut pkg_white_list = HashSet::new();

        pkg_white_list.insert("com.alibaba.android.rimet".into());
        pkg_white_list.insert("com.tencent.mm".into());

        let notify_mode = {
            let hour = chrono::Local::now().hour();
            if hour < 18 && hour > 9 {
                NotifyMode::RemoteDesktop
            } else {
                NotifyMode::Desktop
            }
        };

        Ok(Res::new(NotifyContext {
            inner: Mutex::new(NotifyContextInner {
                pkg_white_list,
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

    pub async fn handle_notification_posted(
        ctx: Res<Self>,
        notification: Notification,
    ) -> Result<(), anyhow::Error> {
        let (notify_mode, cfg) = {
            let ctx = ctx.lock();
            (ctx.notify_mode, ctx.config.clone())
        };

        let do_play = match notify_mode {
            NotifyMode::Desktop => IS_DESKTOP,
            NotifyMode::RemoteDesktop => !IS_DESKTOP,
        };

        if do_play {
            if let Err(e) = Self::play(ctx.clone(), notification) {
                warn!("{:?}", e);
            }
        } else {
            let post_addr = if IS_DESKTOP {
                cfg.mobile_rpc_address
            } else {
                cfg.desktop_rpc_address
            };

            if let Some(addr) = post_addr {
                let mut client = rpc::MtoolAssistantClient::connect(addr).await?;

                let request = tonic::Request::new(rpc::Notification {
                    package_name: notification.package_name,
                });

                _ = client.post_notification(request).await?;
            }
        }

        Ok(())
    }

    pub async fn init(
        ctx: Res<Self>,
        source: Res<SystemEventSource>,
        cmdpal: Res<CommandPalette>,
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

        {
            to_owned![ctx];
            cmdpal.add_top_level_command(CommandItem::from(
                Command::new("Send notification", move || {
                    Self::send_notification(ctx.clone())
                })
                .description("Send notification"),
            ));
        }

        {
            cmdpal
                .add_top_level_command(CommandItem::from(
                    Command::new("Remote desktop mode", {
                        to_owned![ctx];
                        move || {
                            Self::set_notify_mode_with_sync(ctx.clone(), NotifyMode::RemoteDesktop)
                        }
                    })
                    .description("set remote desktop mode"),
                ))
                .add_top_level_command(CommandItem::from(
                    Command::new("Desktop mode", {
                        to_owned![ctx];
                        move || Self::set_notify_mode_with_sync(ctx.clone(), NotifyMode::Desktop)
                    })
                    .description("set desktop mode"),
                ));
        }

        Ok(())
    }

    async fn set_notify_mode_with_sync(
        ctx: Res<Self>,
        mode: NotifyMode,
    ) -> Result<CommandResult, anyhow::Error> {
        ctx.lock().notify_mode = mode;

        Self::with_rpc(ctx, |mut cli| async move {
            _ = cli
                .set_notify_mode(tonic::Request::new(rpc::NotifyModeMessage {
                    mode: match mode {
                        NotifyMode::Desktop => rpc::NotifyMode::DesktopMode.into(),
                        NotifyMode::RemoteDesktop => rpc::NotifyMode::RemoteDesktopMode.into(),
                    },
                }))
                .await?;
            Ok(())
        })
        .await?;

        Ok(CommandResult::Dismiss)
    }

    pub fn set_notify_mode(&self, mode: NotifyMode) {
        self.lock().notify_mode = mode;
    }

    pub async fn toggle_notify_mode_with_sync(ctx: Res<Self>) -> Result<NotifyMode, anyhow::Error> {
        let mode = match ctx.lock().notify_mode {
            NotifyMode::Desktop => NotifyMode::RemoteDesktop,
            NotifyMode::RemoteDesktop => NotifyMode::Desktop,
        };

        Self::set_notify_mode_with_sync(ctx, mode).await?;
        Ok(mode)
    }

    pub fn notify_mode(&self) -> NotifyMode {
        self.lock().notify_mode
    }

    async fn with_rpc<F, O>(ctx: Res<Self>, func: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(rpc::MtoolAssistantClient<tonic::transport::Channel>) -> O,
        O: Future<Output = Result<(), anyhow::Error>>,
    {
        let config = ctx.lock().config.clone();
        if let Some(addr) = config.mobile_rpc_address {
            func(rpc::MtoolAssistantClient::connect(addr).await?).await?
        }
        Ok(())
    }

    async fn send_notification(ctx: Res<Self>) -> Result<CommandResult, anyhow::Error> {
        Self::with_rpc(ctx, |mut cli| async move {
            _ = cli
                .post_notification(tonic::Request::new(rpc::Notification {
                    package_name: "com.tencent.mm".to_owned(),
                }))
                .await?;
            Ok(())
        })
        .await?;

        Ok(CommandResult::Dismiss)
    }

    fn play(ctx: Res<Self>, notification: Notification) -> Result<(), anyhow::Error> {
        info!("{:?}", notification);

        tokio::task::spawn_blocking(move || {
            {
                let mut ctx = ctx.lock();
                if !ctx.pkg_white_list.contains(&notification.package_name) {
                    return Ok(());
                }

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
