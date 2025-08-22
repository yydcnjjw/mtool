use mapp::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct MediaMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub pic_url: String,
    pub duration: u64,
}
