use dioxus::prelude::*;
use mapp::{
    anyhow::{self, Context},
    prelude::*,
    rand::{seq::SliceRandom, thread_rng},
    tokio::{self},
    tokio_stream::StreamExt,
    tracing::{debug, warn},
};
use mtool_p2p::SubjectMessage;
#[cfg(feature = "desktop")]
use mtool_system::NotificationContent;
use mtool_system::{Notification, RemoteSystemEventSource, SystemEvent};
use std::{
    io::{BufReader, Cursor},
    sync::atomic::{AtomicBool, Ordering},
};

pub(crate) struct NotifyReceiver {
    is_playing: AtomicBool,
}

impl NotifyReceiver {
    pub async fn construct() -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self {
            is_playing: AtomicBool::new(false),
        }))
    }
}

impl NotifyReceiver {
    async fn handle_im_notification(receiver: Res<Self>) -> Result<(), anyhow::Error> {
        if cfg!(feature = "mobile") {
            Self::play(receiver)
        } else {
            Ok(())
        }
    }

    pub async fn handle_notification_posted(
        receiver: Res<Self>,
        notification: Notification,
    ) -> Result<(), anyhow::Error> {
        match notification {
            Notification::Im { app: _ } => Self::handle_im_notification(receiver).await,
            #[cfg(feature = "desktop")]
            Notification::Agenda {
                app,
                content: NotificationContent { message, title, .. },
            }
            | Notification::Generic {
                app,
                content: NotificationContent { message, title, .. },
            } => {
                use notify_rust::{Notification, Timeout};
                Notification::new()
                    .app_id("com.yydcnjjw")
                    .appname(&app.id)
                    .summary(&title.unwrap_or_default())
                    .body(&message)
                    .timeout(Timeout::Default)
                    .show()
                    .context("send notification failed")
            }
            _ => Ok(()),
        }
    }

    pub async fn listen_system(
        receiver: Res<Self>,
        source: Res<RemoteSystemEventSource>,
    ) -> Result<(), anyhow::Error> {
        tokio::spawn(async move {
            let mut stream = source.stream();
            while let Some(msg) = stream.next().await {
                to_owned![receiver];
                match msg {
                    Ok(SubjectMessage { data, .. }) => match data.event {
                        SystemEvent::NotificationPosted(notification) => {
                            debug!("SystemEvent::NotificationPosted {notification:?}");
                            tokio::spawn(async move {
                                if let Err(e) =
                                    Self::handle_notification_posted(receiver.clone(), notification)
                                        .await
                                {
                                    warn!("{e:?}");
                                }
                            });
                        }
                        _ => {}
                    },
                    Err(e) => {
                        warn!("{e:?}");
                        break;
                    }
                }
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
            let resp = dioxus_asset_resolver::native::serve_asset(
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
