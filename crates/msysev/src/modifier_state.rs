use crate::{
    keydef::{KeyCode, KeyModifier},
    KeyAction,
};

#[derive(Debug)]
pub struct ModifierState {
    s: KeyModifier,
}

impl ModifierState {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            s: KeyModifier::NONE,
        }
    }

    #[allow(dead_code)]
    pub fn update(&mut self, code: &KeyCode, action: &KeyAction) -> KeyModifier {
        let modifer = match code {
            KeyCode::LeftShift | KeyCode::RightShift => KeyModifier::SHIFT,
            // KeyCode::CapsLock => KeyModifier::CAPSLOCK,
            KeyCode::LeftControl | KeyCode::RightControl => KeyModifier::CONTROL,
            KeyCode::LeftAlt | KeyCode::RightAlt => KeyModifier::ALT,
            KeyCode::NumLock => KeyModifier::NUMLOCK,
            KeyCode::LeftGUI | KeyCode::RightGUI => KeyModifier::SUPER,
            _ => KeyModifier::NONE,
        };

        match action {
            KeyAction::Press => self.s |= modifer,
            KeyAction::Release => self.s -= modifer,
        }
        self.s
    }

    #[allow(dead_code)]
    pub fn get(&self) -> KeyModifier {
        self.s
    }
}
