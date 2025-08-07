use mapp::{
    anyhow,
    prelude::*,
    serde::{Deserialize, Serialize},
    tokio::sync::broadcast,
};

use crate::platform;

#[derive(Clone, Debug)]
pub enum SystenEvent {
    NotificationPosted(Notification),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
#[serde(tag = "type")]
pub enum Notification {
    Generic { app: AppInfo },
    Im { app: AppInfo },
    Media(MediaNotification),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct AppInfo {
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct MediaNotification {
    pub app: AppInfo,
    pub state: PlaybackState,
    pub metadata: MediaMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum MediaState {
    None,
    Stopped,
    Paused,
    Playing,
    FastForwarding,
    Rewinding,
    Buffering,
    Error,
    Connecting,
    SkippingToPrevious,
    SkippingToNext,
    SkippingToQueueItem,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct PlaybackState {
    pub state: MediaState,
    pub position: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct MediaMetadata {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
}

pub struct SystemEventSource {
    pub(crate) inner: platform::SystemEventSource,
}

impl SystemEventSource {
    pub async fn construct(injector: Injector) -> Result<Res<SystemEventSource>, anyhow::Error> {
        Ok(Res::new(SystemEventSource {
            inner: inject_once(&injector, platform::SystemEventSource::new).await??,
        }))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystenEvent> {
        self.inner.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<SystenEvent> {
        self.inner.sender()
    }
}
