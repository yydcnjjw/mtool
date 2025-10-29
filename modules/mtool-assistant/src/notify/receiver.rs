use dioxus::prelude::*;
use mapp::{
    anyhow::{self, anyhow, Context},
    futures::{Stream, StreamExt, TryFutureExt, TryStreamExt},
    prelude::*,
    rand::{seq::SliceRandom, thread_rng},
    sync::Mutex,
    tokio::{self},
    tokio_stream::wrappers::BroadcastStream,
    tracing::warn,
};
use mtool_system::{
    Notification, NotificationContent, RemoteSystemEventSource, SystemEvent, SystemEventSource,
};
use std::io::{BufReader, Cursor};

pub(crate) struct NotifyReceiver {}

impl NotifyReceiver {
    pub async fn construct() -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self {}))
    }
}

impl NotifyReceiver {
    async fn handle_im_notification(_receiver: Res<Self>) -> Result<(), anyhow::Error> {
        if cfg!(feature = "mobile") {
            Self::play();
        }
        Ok(())
    }

    pub async fn handle_notification_posted(
        receiver: Res<Self>,
        notification: Notification,
    ) -> Result<(), anyhow::Error> {
        match notification {
            Notification::Im { app: _ } => Self::handle_im_notification(receiver).await,
            #[allow(unused)]
            Notification::Agenda {
                app,
                content: NotificationContent { message, title, .. },
            }
            | Notification::Generic {
                app,
                content: NotificationContent { message, title, .. },
            } => {
                #[cfg(feature = "desktop")]
                {
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
                #[cfg(not(feature = "desktop"))]
                Ok(())
            }

            _ => Ok(()),
        }
    }

    pub async fn handle_system_event<T>(receiver: Res<Self>, mut stream: T)
    where
        T: Stream<Item = Result<SystemEvent, anyhow::Error>> + Unpin,
    {
        while let Some(event) = stream.next().await {
            to_owned![receiver];
            match event {
                Ok(event) => match event {
                    SystemEvent::NotificationPosted(notification) => {
                        info!(?notification);
                        tokio::spawn(
                            Self::handle_notification_posted(receiver.clone(), notification)
                                .unwrap_or_else(|e| warn!("{e:?}")),
                        );
                    }
                    _ => {}
                },
                Err(e) => {
                    warn!("{e:?}");
                    break;
                }
            }
        }
    }

    pub async fn listen_system_event(
        receiver: Res<Self>,
        source: Res<SystemEventSource>,
    ) -> Result<(), anyhow::Error> {
        tokio::spawn(Self::handle_system_event(
            receiver,
            BroadcastStream::new(source.subscribe()).map_err(|e| anyhow!("{e:?}")),
        ));
        Ok(())
    }

    pub async fn listen_remote_system_event(
        receiver: Res<Self>,
        source: Res<RemoteSystemEventSource>,
    ) -> Result<(), anyhow::Error> {
        tokio::spawn(Self::handle_system_event(
            receiver,
            source.stream().map_ok(|msg| msg.data.event),
        ));
        Ok(())
    }

    fn play() {
        tokio::task::spawn_blocking(move || {
            Self::play_blocking().unwrap_or_else(|e| warn!("{e:?}"))
        });
    }

    fn play_blocking() -> Result<(), anyhow::Error> {
        static LOCK: Mutex<()> = Mutex::new(());
        let _guard = if let Some(guard) = LOCK.try_lock() {
            guard
        } else {
            return Ok(());
        };

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
        Ok(())
    }
}
