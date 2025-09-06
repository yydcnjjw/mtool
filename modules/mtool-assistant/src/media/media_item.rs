use mapp::{
    anyhow::{self, Context},
    async_recursion::async_recursion,
    serde::{Serialize, Serializer},
    url::Url,
};

use super::{MediaMetadata, MediaSource, TimedMetadataSource};

#[derive(Debug, Clone)]
pub struct MediaItem {
    pub id: String,
    pub metadata: Option<MediaMetadata>,

    pub source: MediaSource,
    pub timed_metadata_sources: Vec<TimedMetadataSource>,
}

impl Serialize for MediaItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        #[serde(crate = "mapp::serde")]
        struct Extended<'a> {
            id: &'a String,
            metadata: &'a Option<MediaMetadata>,
            source_uri: String,
            timed_metadata_source_uri_list: Vec<String>,
        }
        Ok(Extended {
            id: &self.id,
            metadata: &self.metadata,
            source_uri: self.source_uri(),
            timed_metadata_source_uri_list: self.timed_metadata_source_uri_list(),
        }
        .serialize(serializer)?)
    }
}

impl MediaItem {
    pub fn new(id: String, source: MediaSource) -> Self {
        Self {
            id,
            source,
            metadata: None,
            timed_metadata_sources: Vec::new(),
        }
    }

    pub fn source_uri(&self) -> String {
        match &self.source {
            MediaSource::Vendor(_) => format!("mtool://{}/audio", self.id),
            MediaSource::Uri(uri) => uri.clone(),
        }
    }

    pub fn timed_metadata_source_uri_list(&self) -> Vec<String> {
        self.timed_metadata_sources
            .iter()
            .map(|TimedMetadataSource { id, .. }| {
                format!("mtool://{}/timed-metadata?id={id}", self.id)
            })
            .collect()
    }

    pub fn with_timed_metadata_source(mut self, source: TimedMetadataSource) -> Self {
        self.timed_metadata_sources.push(source);
        self
    }

    #[allow(unused)]
    #[async_recursion]
    pub async fn resolve_uri(&self, uri: Url) -> Result<String, anyhow::Error> {
        match uri.path() {
            path if path == "/audio" => self.source.resolve_uri().await,
            path if path == "/timed-metadata" => {
                let mut querys = uri.query_pairs();
                if let Some(id) = querys
                    .find(|(k, _)| k == "id")
                    .map(|(_, id)| id.to_string())
                {
                    self.timed_metadata_sources
                        .iter()
                        .find(|track| track.id == id)
                        .context(id)?
                        .resolve_uri()
                        .await
                } else {
                    unreachable!()
                }
            }
            path => unreachable!("{path}"),
        }
    }

    pub fn with_metadata(mut self, metadata: MediaMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}
