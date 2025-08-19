use std::pin::Pin;

use mapp::{
    anyhow,
    futures::Stream,
    prelude::*,
    serde::{Deserialize, Serialize},
    tokio::sync::broadcast::error::RecvError,
};

pub type PlayerEventStream = Pin<Box<dyn Stream<Item = Result<PlayerEvent, RecvError>> + Send>>;

#[async_trait]
pub trait Player {
    async fn play(&self) -> Result<(), anyhow::Error>;

    async fn pause(&self) -> Result<(), anyhow::Error>;

    async fn volume(&self) -> Result<f64, anyhow::Error>;

    async fn set_volume(&self, value: f64) -> Result<(), anyhow::Error>;

    async fn add_media_items(&self, items: Vec<MediaItem>) -> Result<(), anyhow::Error>;

    async fn listen(&self) -> Result<PlayerEventStream, anyhow::Error>;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct MediaItem {
    pub uri: String,
    pub metadata: Option<MediaMetadata>,
}

impl MediaItem {
    pub fn new(uri: String) -> Self {
        Self {
            uri,
            metadata: None,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct MediaMetadata {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum PlayerEvent {
    MediaMetadataChanged(MediaMetadata),
}
