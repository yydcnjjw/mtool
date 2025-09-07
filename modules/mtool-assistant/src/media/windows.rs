use dioxus::hooks::to_owned;
use mapp::{
    anyhow::{self, anyhow, bail, Context},
    async_recursion::async_recursion,
    dashmap::DashMap,
    futures::{StreamExt, TryFutureExt, TryStreamExt},
    itertools::Itertools,
    prelude::*,
    tokio::{self, sync::broadcast},
    tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream},
    tracing::{info, warn},
};
use mtool_dioxus::prelude::*;
use std::sync::Arc;
use windows::{
    core::{h, IInspectable, Interface, HRESULT, HSTRING},
    Foundation::{
        Collections::IVectorChangedEventArgs, IPropertyValue, PropertyValue, TimeSpan,
        TypedEventHandler, Uri,
    },
    Media::{
        Core::{
            MediaBinder, MediaBindingEventArgs, MediaCueEventArgs,
            MediaSource as NativeMediaSource, TimedMetadataKind,
            TimedMetadataTrack as NativeTimedMetadataTrack, TimedMetadataTrackErrorCode,
            TimedMetadataTrackFailedEventArgs, TimedTextCue, TimedTextLine, TimedTextSource,
            TimedTextSourceResolveResultEventArgs,
        },
        Playback::{
            CurrentMediaPlaybackItemChangedEventArgs, MediaPlaybackItem,
            MediaPlaybackItemFailedEventArgs, MediaPlaybackList, MediaPlaybackSession,
            MediaPlaybackState, MediaPlayer as NativeMediaPlayer,
            TimedMetadataTrackPresentationMode,
        },
    },
    Storage::Streams::{DataWriter, InMemoryRandomAccessStream, RandomAccessStreamReference},
};

use super::{
    MediaItem, MediaMetadata, MediaSource, PlaybackState, Player, PlayerEvent, PlayerEventStream,
    TimedCue, TimedCueData, TimedMetadataSource, TimedMetadataTrack, TimedRawTrack,
};

pub struct MediaPlayer {
    player: NativeMediaPlayer,
    sender: broadcast::Sender<PlayerEvent>,
    playback_list: MediaPlaybackList,
    media_items: Arc<DashMap<String, MediaItem>>,
}

impl Drop for MediaPlayer {
    fn drop(&mut self) {
        _ = self.player.Close();
    }
}

#[async_trait]
impl Player for MediaPlayer {
    async fn play(&self) -> Result<(), anyhow::Error> {
        Ok(self.player.Play()?)
    }

    async fn pause(&self) -> Result<(), anyhow::Error> {
        Ok(self.player.Pause()?)
    }

    async fn playback_state(&self) -> Result<PlaybackState, anyhow::Error> {
        self.player.PlaybackSession()?.PlaybackState()?.try_into()
    }

    async fn volume(&self) -> Result<f64, anyhow::Error> {
        Ok(self.player.Volume()?)
    }

    async fn set_volume(&self, value: f64) -> Result<(), anyhow::Error> {
        Ok(self.player.SetVolume(value)?)
    }

    async fn current_media_item(&self) -> Result<Option<MediaItem>, anyhow::Error> {
        match self.playback_list.CurrentItem().or_else(|e| {
            if e.code() == HRESULT(0) {
                self.playback_list.StartingItem()
            } else {
                Err(e)
            }
        }) {
            Ok(item) => Ok(Some(Self::get_media_item(item, self.media_items.clone())?)),
            Err(e) => {
                if e.code() == HRESULT(0) {
                    Ok(None)
                } else {
                    Err(anyhow!("{e:?}"))
                }
            }
        }
    }

    async fn set_media_items(&self, items: Vec<MediaItem>) -> Result<(), anyhow::Error> {
        self.media_items.clear();

        for item in items {
            let MediaItem {
                id,
                source,
                timed_metadata_sources,
                metadata,
                ..
            } = &item;

            {
                let source =
                    Self::create_native_media_source(source, timed_metadata_sources).await?;

                {
                    let cp = source.CustomProperties()?;
                    cp.Insert(h!("id"), &PropertyValue::CreateString(&HSTRING::from(id))?)?;
                }

                let item = MediaPlaybackItem::Create(&source)?;

                let props = item.GetDisplayProperties()?;
                props.SetType(windows::Media::MediaPlaybackType::Music)?;

                if let Some(MediaMetadata {
                    title,
                    artist,
                    album,
                    pic_url,
                    duration: _,
                }) = metadata
                {
                    let music = props.MusicProperties()?;
                    music.SetTitle(&HSTRING::from(title))?;
                    music.SetArtist(&HSTRING::from(artist))?;
                    music.SetAlbumTitle(&HSTRING::from(album))?;

                    props.SetThumbnail(&RandomAccessStreamReference::CreateFromUri(
                        &Uri::CreateUri(&HSTRING::from(pic_url))?,
                    )?)?;
                }

                item.ApplyDisplayProperties(&props)?;

                item.TimedMetadataTracksChanged(&self.on_timed_metadata_tracks_changed())?;

                self.playback_list.Items()?.Append(&item)?;
            }

            self.media_items.insert(id.clone(), item);
        }

        self.player.SetSource(&self.playback_list)?;

        Ok(())
    }

    async fn listen(&self) -> Result<PlayerEventStream, anyhow::Error> {
        Ok(BroadcastStream::new(self.sender.subscribe())
            .map_err(|e| match e {
                BroadcastStreamRecvError::Lagged(value) => {
                    broadcast::error::RecvError::Lagged(value)
                }
            })
            .boxed())
    }
}

impl MediaPlayer {
    pub async fn new(_context: DioxusContext) -> Result<Self, anyhow::Error> {
        let (sender, _) = broadcast::channel(16);
        let media_items = Arc::new(DashMap::new());

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
                    if let Some(args) = args.cloned() {
                        warn!("{:?}: {:?}", args.Item()?.Source()?.Uri()?, args.Error()?);
                    }
                    Ok(())
                }))?;
            }

            {
                to_owned![sender, media_items];
                playlist.CurrentItemChanged(&TypedEventHandler::<
                    MediaPlaybackList,
                    CurrentMediaPlaybackItemChangedEventArgs,
                >::new(move |_, args| {
                    if let Some(args) = args.as_ref() {
                        let item = args.NewItem()?;

                        _ = Self::get_media_item(item, media_items.clone()).and_then(|item| {
                            if let Some(metadata) = item.metadata.clone() {
                                if let Err(e) =
                                    sender.send(PlayerEvent::MediaMetadataChanged { metadata })
                                {
                                    warn!("{e:?}, receiver_count: {}", sender.receiver_count());
                                }
                            }
                            Ok(())
                        });
                    }

                    Ok(())
                }))?;
            }

            {
                to_owned![sender];
                player
                    .PlaybackSession()?
                    .PlaybackStateChanged(
                        &TypedEventHandler::<MediaPlaybackSession, IInspectable>::new(
                            move |session, _| {
                                if let Some(session) = session.as_ref() {
                                    if let Err(e) = sender.send(PlayerEvent::PlaybackStateChanged {
                                        state: session.PlaybackState()?.try_into().unwrap(),
                                    }) {
                                        warn!("{e:?}, receiver_count: {}", sender.receiver_count());
                                    }
                                }

                                Ok(())
                            },
                        ),
                    )?;
            }

            playlist
        };

        Ok(Self {
            player,
            sender,
            playback_list,
            media_items,
        })
    }

    async fn create_native_media_source(
        source: &MediaSource,
        timed_metadata_sources: &Vec<TimedMetadataSource>,
    ) -> Result<NativeMediaSource, anyhow::Error> {
        let timed_metadata_sources = timed_metadata_sources.iter().cloned().collect_vec();

        let source = match source {
            MediaSource::Vendor(callback) => {
                to_owned![callback];
                let rt = tokio::runtime::Handle::current();
                let handler = TypedEventHandler::<MediaBinder, MediaBindingEventArgs>::new(
                    move |binder, args| {
                        let binder = binder.cloned().expect("binder");
                        let args = args.cloned().expect("args");

                        let defer = args.GetDeferral()?;

                        to_owned![callback, timed_metadata_sources];

                        rt.spawn(
                            async move {
                                match callback().await? {
                                    MediaSource::Uri(uri) => {
                                        args.SetUri(&Uri::CreateUri(&HSTRING::from(&uri))?)?
                                    }
                                    _ => unreachable!(),
                                };

                                let source = binder.Source()?;
                                for timed_metadata_source in timed_metadata_sources {
                                    Self::add_timed_metadata_source(
                                        &source,
                                        &timed_metadata_source,
                                    )
                                    .await?;
                                }

                                _ = defer.Complete();
                                Ok::<_, anyhow::Error>(())
                            }
                            .unwrap_or_else(|e| {
                                warn!("{e:?}");
                                ()
                            }),
                        );

                        Ok(())
                    },
                );

                let binder = MediaBinder::new()?;
                binder.Binding(&handler)?;
                NativeMediaSource::CreateFromMediaBinder(&binder)?
            }
            MediaSource::Uri(uri) => {
                NativeMediaSource::CreateFromUri(&Uri::CreateUri(&HSTRING::from(uri))?)?
            }
        };

        Ok(source)
    }

    #[async_recursion]
    async fn add_timed_metadata_source(
        source: &NativeMediaSource,
        TimedMetadataSource { id, track }: &TimedMetadataSource,
    ) -> Result<(), anyhow::Error> {
        match track {
            TimedMetadataTrack::Vendor(callback) => {
                return Self::add_timed_metadata_source(source, &callback().await?).await;
            }
            TimedMetadataTrack::Subtitle(subtitle) => {
                let text = subtitle.to_string();

                let stream = InMemoryRandomAccessStream::new()?;

                let writer = DataWriter::CreateDataWriter(&stream.GetOutputStreamAt(0)?)?;
                writer.WriteString(&HSTRING::from(text))?;
                writer.StoreAsync()?.await?;
                writer.FlushAsync()?.await?;
                writer.DetachStream()?;

                let track = TimedTextSource::CreateFromStream(&stream)?;
                to_owned![id];
                track.Resolved(&TypedEventHandler::<
                    TimedTextSource,
                    TimedTextSourceResolveResultEventArgs,
                >::new(move |_, args| {
                    let args = args.as_ref().expect("args");
                    if args.Error()?.ErrorCode()? != TimedMetadataTrackErrorCode::None {
                        let err = args.Error()?;
                        warn!("{:?}, {}", err.ErrorCode()?, err.ExtendedError()?.message());
                    }

                    for track in args.Tracks()? {
                        track.SetLabel(&HSTRING::from(&id))?;
                    }

                    Ok(())
                }))?;
                source.ExternalTimedTextSources()?.Append(&track)?;
            }
            TimedMetadataTrack::Raw(TimedRawTrack { cues }) => {
                info!("add timed raw track");
                let track = NativeTimedMetadataTrack::Create(
                    &HSTRING::from(id),
                    &HSTRING::from(id),
                    TimedMetadataKind::Subtitle,
                )?;

                track.SetLabel(&HSTRING::from(id))?;

                for cue in cues {
                    track.AddCue(&TimedTextCue::try_from(cue)?)?;
                }

                source.ExternalTimedMetadataTracks()?.Append(&track)?;
            }
        };

        Ok(())
    }

    fn on_timed_metadata_tracks_changed(
        &self,
    ) -> TypedEventHandler<MediaPlaybackItem, IVectorChangedEventArgs> {
        to_owned![self.sender];

        TypedEventHandler::<MediaPlaybackItem, IVectorChangedEventArgs>::new(move |item, args| {
            let item = item.as_ref().expect("item");
            if let Some(args) = args.as_ref() {
                let tracks = item.TimedMetadataTracks()?;
                tracks.SetPresentationMode(
                    args.Index()?,
                    TimedMetadataTrackPresentationMode::PlatformPresented,
                )?;

                let track = tracks.GetAt(args.Index()?)?;

                to_owned![sender];
                track.CueEntered(&TypedEventHandler::<
                    NativeTimedMetadataTrack,
                    MediaCueEventArgs,
                >::new(move |track, args| {
                    if let Some(args) = args.as_ref() {
                        let track = track.as_ref().expect("track");
                        let cue = args.Cue()?;

                        TimedCue::try_from(cue.cast::<TimedTextCue>()?)
                            .and_then(|cue| {
                                _ = sender.send(PlayerEvent::TimedCuesChanged {
                                    track_id: track.Label()?.to_string(),
                                    cue,
                                });
                                Ok(())
                            })
                            .unwrap_or_else(|e| warn!("{e:?}"));
                    }
                    Ok(())
                }))?;

                track.TrackFailed(&TypedEventHandler::<
                    NativeTimedMetadataTrack,
                    TimedMetadataTrackFailedEventArgs,
                >::new(move |_, args| {
                    if let Some(args) = args.as_ref() {
                        let err = args.Error()?;
                        warn!(
                            "code: {}, result: {}",
                            err.ErrorCode()?.0,
                            err.ExtendedError()?.message()
                        );
                    }
                    Ok(())
                }))?;
            }

            Ok(())
        })
    }

    fn get_media_item(
        item: MediaPlaybackItem,
        items: Arc<DashMap<String, MediaItem>>,
    ) -> Result<MediaItem, anyhow::Error> {
        let id = item
            .Source()?
            .CustomProperties()?
            .Lookup(h!("id"))?
            .cast::<IPropertyValue>()?
            .GetString()?
            .to_string();
        items
            .get(&id)
            .map(|item| item.clone())
            .context(format!("{id} not found"))
    }
}

impl TryFrom<&TimedCue> for TimedTextCue {
    type Error = anyhow::Error;

    fn try_from(
        TimedCue {
            id,
            data,
            start_time,
            duration,
        }: &TimedCue,
    ) -> Result<Self, Self::Error> {
        let cue = TimedTextCue::new()?;

        if let Some(id) = id {
            cue.SetId(&HSTRING::from(id))?;
        }

        let line = TimedTextLine::new()?;
        let text = match data {
            TimedCueData::Text(text) => text.as_str(),
            TimedCueData::Binary(data) => str::from_utf8(&data)?,
        };
        line.SetText(&HSTRING::from(text))?;
        cue.Lines()?.Append(&line)?;

        cue.SetStartTime(TimeSpan::from(start_time.clone()))?;

        cue.SetDuration(TimeSpan::from(duration.clone()))?;

        Ok(cue)
    }
}

impl TryFrom<TimedTextCue> for TimedCue {
    type Error = anyhow::Error;

    fn try_from(cue: TimedTextCue) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Some(cue.Id()?.to_string()),
            data: TimedCueData::Text(
                cue.Lines()?
                    .GetView()?
                    .into_iter()
                    .map(|line| line.Text().unwrap_or_default())
                    .join("\n"),
            ),
            start_time: cue.StartTime()?.into(),
            duration: cue.Duration()?.into(),
        })
    }
}

impl TryFrom<MediaPlaybackState> for PlaybackState {
    type Error = anyhow::Error;

    fn try_from(value: MediaPlaybackState) -> Result<Self, Self::Error> {
        Ok(match value {
            MediaPlaybackState::None => Self::None,
            MediaPlaybackState::Opening => Self::Opening,
            MediaPlaybackState::Buffering => Self::Buffering,
            MediaPlaybackState::Playing => Self::Playing,
            MediaPlaybackState::Paused => Self::Paused,
            value => bail!("Unknown MediaPlaybackState: {}", value.0),
        })
    }
}
