use std::sync::Arc;

use dioxus::hooks::to_owned;
use mapp::{
    anyhow,
    prelude::*,
    tokio::{self, sync::mpsc},
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

use crate::music::{Netease, Playlist};

#[derive(Clone)]
pub struct MediaPlayer {
    player: Arc<NativeMediaPlayer>,
}

impl MediaPlayer {
    pub async fn new(_context: Res<DioxusContext>) -> Result<Self, anyhow::Error> {
        let player = NativeMediaPlayer::new()?;

        Ok(Self {
            player: Arc::new(player),
        })
    }

    pub fn play(&self) -> Result<(), anyhow::Error> {
        Ok(self.player.Play()?)
    }

    pub fn pause(&self) -> Result<(), anyhow::Error> {
        Ok(self.player.Pause()?)
    }

    pub fn volume(&self) -> Result<usize, anyhow::Error> {
        Ok((self.player.Volume()?.clamp(0., 1.) * 100.).round() as usize)
    }

    pub fn set_volume(&self, value: usize) -> Result<(), anyhow::Error> {
        Ok(self.player.SetVolume(value as f64 / 100.)?)
    }

    pub fn add_playlist(&self, netease_playlist: Playlist) -> Result<(), anyhow::Error> {
        let playlist = MediaPlaybackList::new()?;

        let rt = tokio::runtime::Handle::current();

        for song in netease_playlist.entries {
            let binder = MediaBinder::new()?;
            binder.SetToken(&HSTRING::from(&song.url))?;
            binder.Binding(
                &TypedEventHandler::<MediaBinder, MediaBindingEventArgs>::new({
                    to_owned![rt];
                    move |_, args| {
                        if let Some(args) = args {
                            let defer = args.GetDeferral()?;
                            to_owned![song.url, args];
                            info!("get song: {url}");
                            rt.spawn_blocking(move || {
                                match Netease::new().get_song(url) {
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
            playlist.Items()?.Append(&item)?;
        }

        playlist.SetShuffleEnabled(true)?;
        playlist.ItemFailed(&TypedEventHandler::<
            MediaPlaybackList,
            MediaPlaybackItemFailedEventArgs,
        >::new(move |_, args| {
            if let Some(args) = args {
                warn!("{:?}: {:?}", args.Item()?.Source()?.Uri()?, args.Error()?);
            }
            Ok(())
        }))?;

        self.player.SetSource(&playlist)?;

        Ok(())
    }
}
