use mapp::{
    anyhow,
    futures::Stream,
    prelude::*,
    serde::{Deserialize, Serialize},
    tokio::sync::broadcast::error::RecvError,
};
use std::pin::Pin;

use super::{MediaItem, MediaMetadata, TimedCue};

pub type PlayerEventStream = Pin<Box<dyn Stream<Item = Result<PlayerEvent, RecvError>> + Send>>;

#[async_trait]
pub trait Player {
    async fn play(&self) -> Result<(), anyhow::Error>;

    async fn pause(&self) -> Result<(), anyhow::Error>;

    async fn playback_state(&self) -> Result<PlaybackState, anyhow::Error>;

    #[allow(unused)]
    async fn volume(&self) -> Result<f64, anyhow::Error>;

    async fn set_volume(&self, value: f64) -> Result<(), anyhow::Error>;

    async fn set_media_items(&self, items: Vec<MediaItem>) -> Result<(), anyhow::Error>;

    async fn current_media_item(&self) -> Result<Option<MediaItem>, anyhow::Error>;

    async fn listen(&self) -> Result<PlayerEventStream, anyhow::Error>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum PlayerEvent {
    MediaMetadataChanged { metadata: MediaMetadata },
    TimedCuesChanged { track_id: String, cue: TimedCue },
    PlaybackStateChanged { state: PlaybackState },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum PlaybackState {
    None,
    Opening,
    Buffering,
    Playing,
    Paused,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self::None
    }
}
