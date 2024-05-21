use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    pub command: String,
    pub kbd: String,
    pub when: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HotkeyMap(pub Vec<Hotkey>);

impl Deref for HotkeyMap {
    type Target = Vec<Hotkey>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
