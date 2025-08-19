use dioxus::prelude::*;
use mapp::{
    anyhow::{self, Context},
    prelude::*,
    rand::{seq::SliceRandom, thread_rng},
    serde_json,
    tokio::{self},
    tracing::{debug, warn},
};
use mtool_system::{AppInfo, Notification, SystemEventSource, SystenEvent};
use std::{
    io::{BufReader, Cursor},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::context::{AssistantContext, AssistantMode};

use super::rpc::{self, NotifyClient};

pub(crate) struct NotifyReceiver {
    is_playing: AtomicBool,
    context: Res<AssistantContext>,
}

impl NotifyReceiver {
    pub async fn construct(context: Res<AssistantContext>) -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self {
            is_playing: AtomicBool::new(false),
            context,
        }))
    }
}

impl NotifyReceiver {
    async fn post_notification(
        receiver: Res<Self>,
        notification: Notification,
    ) -> Result<(), anyhow::Error> {
        if let Some(request_address) = receiver.context.request_address() {
            let mut client = NotifyClient::connect(request_address).await?;

            let request = tonic::Request::new(rpc::Notification {
                data: serde_json::to_vec(&notification)?,
            });

            _ = client.post_notification(request).await?;
        }
        Ok(())
    }

    async fn handle_im_notification(
        receiver: Res<Self>,
        app: AppInfo,
    ) -> Result<(), anyhow::Error> {
        let ctx = &receiver.context;

        let do_play = match ctx.mode() {
            AssistantMode::Desktop => ctx.is_desktop(),
            AssistantMode::RemoteDesktop => !ctx.is_desktop(),
        };

        if do_play {
            Self::play(receiver)
        } else {
            Self::post_notification(receiver, Notification::Im { app }).await
        }
    }

    pub async fn handle_notification_posted(
        receiver: Res<Self>,
        notification: Notification,
    ) -> Result<(), anyhow::Error> {
        match notification {
            Notification::Im { app } => Self::handle_im_notification(receiver, app).await,
            _ => Ok(()),
        }
    }

    pub async fn listen_system(
        receiver: Res<Self>,
        source: Res<SystemEventSource>,
    ) -> Result<(), anyhow::Error> {
        tokio::spawn(async move {
            let mut rx = source.subscribe();
            while let Ok(ev) = rx.recv().await {
                to_owned![receiver];

                debug!("{:?}", ev);

                tokio::spawn(async move {
                    if let Err(e) = match ev {
                        SystenEvent::NotificationPosted(notification) => {
                            Self::handle_notification_posted(receiver.clone(), notification).await
                        }
                    } {
                        warn!("{:?}", e);
                    }
                });
            }
        });

        Ok(())
    }

    fn play(receiver: Res<Self>) -> Result<(), anyhow::Error> {
        tokio::task::spawn_blocking(move || {
            {
                if receiver.is_playing.load(Ordering::Relaxed) {
                    return Ok(());
                }

                receiver.is_playing.store(true, Ordering::Relaxed);
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

            receiver.is_playing.store(false, Ordering::Relaxed);

            Ok::<_, anyhow::Error>(())
        });
        Ok(())
    }
}
