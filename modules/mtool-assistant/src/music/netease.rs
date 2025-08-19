use mapp::{
    anyhow,
    serde::{Deserialize, Serialize},
    serde_json,
    tracing::warn,
};
use mtool_core::py_run;
use pyo3::prelude::*;

#[derive(Clone)]
pub struct Netease {}

impl Netease {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_playlist(&self, id: String) -> Result<Playlist, anyhow::Error> {
        let playlist = py_run(move |py| {
            let netease = py.import("mtool_assistant.music.netease")?;
            netease
                .getattr("get_toplist")?
                .call1((id,))?
                .extract::<String>()
        })?;
        Ok(serde_json::from_str(&playlist)?)
    }

    pub fn get_song(&self, url: String) -> Result<Song, anyhow::Error> {
        let song = py_run(move |py| {
            let netease = py.import("mtool_assistant.music.netease")?;
            netease
                .getattr("get_song")?
                .call1((url,))?
                .extract::<String>()
        })?;
        Ok(serde_json::from_str(&song).inspect_err(|_| {
            warn!("{song}");
        })?)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct Playlist {
    pub title: String,
    pub description: String,
    pub uploader: String,
    pub uploader_id: String,
    pub timestamp: i64,
    pub id: String,
    pub entries: Vec<PlaylistEntry>,
    pub webpage_url: String,
    pub upload_date: String,
    pub playlist_count: i64,
    pub epoch: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct PlaylistEntry {
    pub id: String,
    pub title: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct Song {
    pub id: String,
    #[serde(default)]
    pub creators: Vec<String>,
    #[serde(default)]
    pub album_artists: Vec<String>,
    pub description: String,
    pub subtitles: Subtitles,
    pub title: String,
    pub thumbnail: String,
    pub duration: i64,
    pub album: String,
    pub average_rating: i64,
    pub webpage_url: String,
    #[serde(default)]
    pub thumbnails: Vec<Thumbnail>,
    pub fulltitle: String,
    pub duration_string: String,
    pub album_artist: String,
    pub creator: String,
    pub epoch: i64,
    pub url: String,
    pub format_id: String,
    pub ext: String,
    pub abr: i64,
    pub filesize: i64,
    pub protocol: String,
    pub audio_ext: String,
    pub video_ext: String,
    pub vbr: i64,
    pub tbr: i64,
    pub acodec: String,
    pub resolution: String,
    pub format: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct Subtitles {
    #[serde(default)]
    pub lyrics_merged: Vec<LyricsMerged>,
    #[serde(default)]
    pub lyrics: Vec<Lyric>,
    #[serde(default)]
    pub lyrics_translated: Vec<LyricsTranslated>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct LyricsMerged {
    pub data: String,
    pub ext: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct Lyric {
    pub data: String,
    pub ext: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct LyricsTranslated {
    pub data: String,
    pub ext: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct Thumbnail {
    pub url: String,
    pub id: String,
}
