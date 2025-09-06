use mapp::{
    anyhow,
    async_recursion::async_recursion,
    futures::future::{BoxFuture, FutureExt},
    serde::{Deserialize, Serialize},
    serde_with::{serde_as, DurationMilliSeconds},
};
use std::{fmt, future::Future, sync::Arc, time::Duration};

use super::SubtitleTrack;

#[derive(Debug, Clone)]
pub struct TimedMetadataSource {
    pub id: String,
    pub track: TimedMetadataTrack,
}

impl TimedMetadataSource {
    pub fn vendor<F, Fut>(id: String, f: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<TimedMetadataSource, anyhow::Error>> + Send + 'static,
    {
        Self {
            id,
            track: TimedMetadataTrack::Vendor(Arc::new(move || f().boxed())),
        }
    }

    pub fn new(id: String, track: TimedMetadataTrack) -> Self {
        Self { id, track }
    }

    #[async_recursion]
    pub async fn resolve_uri(&self) -> Result<String, anyhow::Error> {
        match &self.track {
            TimedMetadataTrack::Vendor(callback) => callback().await?.resolve_uri().await,
            TimedMetadataTrack::Subtitle(subtitle) => Ok(subtitle.data_uri()),
            TimedMetadataTrack::Raw(_) => unimplemented!(),
        }
    }
}

#[allow(unused)]
#[derive(Clone)]
pub enum TimedMetadataTrack {
    Vendor(
        Arc<
            dyn Fn() -> BoxFuture<'static, Result<TimedMetadataSource, anyhow::Error>>
                + Send
                + Sync,
        >,
    ),
    Subtitle(SubtitleTrack),
    Raw(TimedRawTrack),
}

impl fmt::Debug for TimedMetadataTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vendor { .. } => f.debug_tuple("Vendor").finish(),
            Self::Subtitle(arg0) => f.debug_tuple("Subtitle").field(arg0).finish(),
            Self::Raw(TimedRawTrack { .. }) => f.debug_struct("Raw").finish(),
        }
    }
}

#[derive(Clone)]
pub struct TimedRawTrack {
    pub cues: Vec<TimedCue>,
}

#[serde_as(crate = "mapp::serde_with")]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct TimedCue {
    pub id: Option<String>,
    pub data: TimedCueData,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub start_time: Duration,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub duration: Duration,
}

impl Default for TimedCue {
    fn default() -> Self {
        Self {
            id: Default::default(),
            data: Default::default(),
            start_time: Default::default(),
            duration: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum TimedCueData {
    Text(String),
    Binary(Vec<u8>),
}

impl ToString for TimedCueData {
    fn to_string(&self) -> String {
        match self {
            TimedCueData::Text(text) => text.clone(),
            TimedCueData::Binary(_) => String::new(),
        }
    }
}

impl Default for TimedCueData {
    fn default() -> Self {
        Self::Text(Default::default())
    }
}
