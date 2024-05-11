use mime::Mime;

use crate::keyboard::*;

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Selection(SelectionEvent),
    Exit,
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: PhysicalKey,
    pub modifiers: ModifierState,
    pub state: KeyState,
}

#[derive(Debug, Clone)]
pub struct SelectionEvent {
    pub data: Vec<u8>,
    pub mime_type: Mime,
}
