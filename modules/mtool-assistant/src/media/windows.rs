use dioxus::hooks::to_owned;
use mapp::{
    anyhow,
    futures::TryStreamExt,
    prelude::*,
    tokio::{self, sync::broadcast},
    tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream},
    tracing::{info, warn},
};
use mtool_dioxus::prelude::*;
use windows::{
    core::HSTRING,
    Foundation::{TypedEventHandler, Uri},
    Media::{
        Core::{MediaBinder, MediaBindingEventArgs, MediaSource},
        Playback::{
            MediaPlaybackItem, MediaPlaybackItemFailedEventArgs, MediaPlaybackList,
            MediaPlayer as NativeMediaPlayer,
        },
    },
};

use super::{MediaItem, Netease, Player, PlayerEvent, PlayerEventStream};

pub struct MediaPlayer {
    player: NativeMediaPlayer,
    sender: broadcast::Sender<PlayerEvent>,
    playback_list: MediaPlaybackList,
}

#[async_trait]
impl Player for MediaPlayer {
    async fn play(&self) -> Result<(), anyhow::Error> {
        Ok(self.player.Play()?)
    }

    async fn pause(&self) -> Result<(), anyhow::Error> {
        Ok(self.player.Pause()?)
    }

    async fn volume(&self) -> Result<f64, anyhow::Error> {
        Ok(self.player.Volume()?)
    }

    async fn set_volume(&self, value: f64) -> Result<(), anyhow::Error> {
        Ok(self.player.SetVolume(value)?)
    }

    async fn add_media_items(&self, items: Vec<MediaItem>) -> Result<(), anyhow::Error> {
        let rt = tokio::runtime::Handle::current();

        for MediaItem { uri, .. } in items {
            let binder = MediaBinder::new()?;
            binder.SetToken(&HSTRING::from(&uri))?;
            binder.Binding(
                &TypedEventHandler::<MediaBinder, MediaBindingEventArgs>::new({
                    to_owned![rt];
                    move |_, args| {
                        if let Some(args) = args {
                            let defer = args.GetDeferral()?;
                            to_owned![uri, args];
                            info!("get song: {uri}");

                            rt.spawn_blocking(move || {
                                match Netease::new().get_song(uri) {
                                    Ok(song) => {
                                        _ = args.SetUri(
                                            &Uri::CreateUri(&HSTRING::from(&song.url)).unwrap(),
                                        );
                                    }
                                    Err(e) => {
                                        warn!("{:?}", e);
                                    }
                                }
                                _ = defer.Complete();
                            });
                        }

                        Ok(())
                    }
                }),
            )?;
            let source = MediaSource::CreateFromMediaBinder(&binder)?;
            let item = MediaPlaybackItem::Create(&source)?;
            self.playback_list.Items()?.Append(&item)?;
        }

        self.player.SetSource(&self.playback_list)?;
        Ok(())
    }

    async fn listen(&self) -> Result<PlayerEventStream, anyhow::Error> {
        Ok(Box::pin(
            BroadcastStream::new(self.sender.subscribe()).map_err(|e| match e {
                BroadcastStreamRecvError::Lagged(value) => {
                    broadcast::error::RecvError::Lagged(value)
                }
            }),
        ) as PlayerEventStream)
    }
}

impl MediaPlayer {
    pub async fn new(_context: Res<DioxusContext>) -> Result<Self, anyhow::Error> {
        let player = NativeMediaPlayer::new()?;
        let playback_list = {
            let playlist = MediaPlaybackList::new()?;
            playlist.SetShuffleEnabled(true)?;
            playlist.AutoRepeatEnabled()?;

            {
                playlist.ItemFailed(&TypedEventHandler::<
                    MediaPlaybackList,
                    MediaPlaybackItemFailedEventArgs,
                >::new(move |_, args| {
                    if let Some(args) = args {
                        warn!("{:?}: {:?}", args.Item()?.Source()?.Uri()?, args.Error()?);
                    }
                    Ok(())
                }))?;
            }

            playlist
        };

        let (sender, _) = broadcast::channel(64);

        Ok(Self {
            player,
            sender,
            playback_list,
        })
    }
}
