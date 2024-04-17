use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    pub command: String,
    pub kbd: String,
    pub when: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyMap(pub Vec<Hotkey>);
