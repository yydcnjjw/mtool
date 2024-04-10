use serde::{Deserialize, Serialize};
use serde_with::{serde_as, KeyValueMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    #[serde(rename = "$key$")]
    pub command: String,
    pub kbd: String,
    pub when: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyMap(#[serde_as(as = "KeyValueMap<_>")] pub Vec<Hotkey>);
