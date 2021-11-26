use crate::keydef::{KeyCode, KeyModifier};

#[derive(Debug, Clone)]
pub enum KeyAction {
    Press,
    Release,
}

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub scancode: u32,
    pub keycode: KeyCode,
    pub modifiers: KeyModifier,
    pub action: KeyAction,
}

pub type EventSender = broadcast::Sender<Event>;
