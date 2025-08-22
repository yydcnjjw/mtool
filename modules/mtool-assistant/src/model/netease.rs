use std::sync::Arc;

use dioxus::hooks::to_owned;
use mapp::{
    anyhow::{self, Context},
    futures::{future::try_join_all, FutureExt, TryFutureExt},
    tokio::sync::OnceCell,
    tracing::info,
};
use mcloud_api::netease::{Lyrics, MusicApi, SongInfo};

use crate::media::{
    MediaItem, MediaMetadata, MediaSource, SubtitleTrack, TimedMetadataSource, TimedMetadataTrack,
    TimedRawTrack,
};

#[derive(Clone)]
pub struct NeteaseViewModel {
    api: MusicApi,
}

impl NeteaseViewModel {
    pub fn new() -> Self {
        Self {
            api: MusicApi::new(5),
        }
    }

    pub async fn load_media_items_from_playlist(
        &self,
        song_list_ids: &[u64],
    ) -> Result<Vec<MediaItem>, anyhow::Error> {
        Ok(try_join_all(song_list_ids.iter().cloned().map(|id| {
            self.api
                .song_list_detail(id)
                .map_ok(|song_list| song_list.songs)
        }))
        .await?
        .into_iter()
        .flatten()
        .map(
            |SongInfo {
                 id,
                 name,
                 singer,
                 album,
                 pic_url,
                 duration,
                 ..
             }| {
                let mut item = MediaItem::new(
                    id.to_string(),
                    MediaSource::vendor({
                        to_owned![self.api];
                        move || {
                            to_owned![api];

                            info!("get song {id}");

                            async move {
                                api.songs_url(&[id], "1900000")
                                    .await
                                    .and_then(|songs| {
                                        Ok(songs.first().cloned().context("song not found")?.url)
                                    })
                                    .map(|uri| MediaSource::Uri(uri))
                            }
                        }
                    }),
                );

                {
                    let song_lyrics = Arc::new(OnceCell::new());

                    #[derive(Clone, Copy)]
                    enum LyricType {
                        Origin,
                        Translate,
                    }

                    let callback = |ty: LyricType| {
                        to_owned![self.api, song_lyrics];

                        move || {
                            to_owned![api, song_lyrics];

                            info!("get song lyric {id}");

                            async move {
                                let Lyrics { lyric, tlyric } = song_lyrics
                                    .get_or_try_init(
                                        move || async move { api.song_lyric(id).await },
                                    )
                                    .await?;

                                Ok::<_, anyhow::Error>(match ty {
                                    LyricType::Origin => TimedMetadataSource::new(
                                        "origin".into(),
                                        TimedMetadataTrack::Subtitle(SubtitleTrack::from_lrc(
                                            lyric, duration,
                                        )?),
                                    ),
                                    LyricType::Translate => TimedMetadataSource::new(
                                        "translate".into(),
                                        TimedMetadataTrack::Subtitle(SubtitleTrack::from_lrc(
                                            tlyric.as_ref().unwrap_or(lyric),
                                            duration,
                                        )?),
                                    ),
                                })
                            }
                        }
                    };

                    item = item
                        .with_timed_metadata_source(TimedMetadataSource::vendor(
                            "origin".into(),
                            callback(LyricType::Origin),
                        ))
                        .with_timed_metadata_source(TimedMetadataSource::vendor(
                            "translate".into(),
                            callback(LyricType::Translate),
                        ));
                }

                item.with_metadata(MediaMetadata {
                    title: name,
                    artist: singer,
                    album,
                    pic_url,
                    duration,
                })
            },
        )
        .collect())
    }
}
