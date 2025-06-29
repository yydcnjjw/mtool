use std::ops::Deref;

use mapp::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct Hotkey {
    pub command: String,
    pub kbd: String,
    pub when: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct HotkeyMap(pub Vec<Hotkey>);

impl Deref for HotkeyMap {
    type Target = Vec<Hotkey>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
