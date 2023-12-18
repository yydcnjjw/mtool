use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq)]
pub struct PdfFile {
    pub path: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageInfo {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PdfDocumentInfo {
    pub pages: Vec<PageInfo>,
}

impl PdfDocumentInfo {
    pub fn width(&self) -> usize {
        self.pages.iter().map(|size| size.width).max().unwrap_or(0)
    }

    pub fn height(&self) -> usize {
        self.pages.iter().map(|size| size.height).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleEvent {
    pub scale: f32,
    pub mouse_point: (i32, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollEvent {
    pub left: i32,
    pub top: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position<T> {
    pub x: T,
    pub y: T,
}

impl<T> Position<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WMouseEvent {
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
    pub button: i16,
    pub buttons: u16,
    pub client_offset: Position<i32>,
    pub layer_offset: Position<i32>,
    pub movement_offset: Position<i32>,
    pub offset: Position<i32>,
    pub page_offset: Position<i32>,
    pub screen_offset: Position<i32>,
}

impl From<web_sys::MouseEvent> for WMouseEvent {
    fn from(v: web_sys::MouseEvent) -> Self {
        Self {
            alt_key: v.alt_key(),
            ctrl_key: v.ctrl_key(),
            meta_key: v.meta_key(),
            shift_key: v.shift_key(),
            button: v.button(),
            buttons: v.buttons(),
            client_offset: Position::new(v.client_x(), v.client_y()),
            layer_offset: Position::new(v.layer_x(), v.layer_y()),
            movement_offset: Position::new(v.movement_x(), v.movement_y()),
            offset: Position::new(v.offset_x(), v.offset_y()),
            page_offset: Position::new(v.page_x(), v.page_y()),
            screen_offset: Position::new(v.screen_x(), v.screen_y()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseEvent {
    Up(WMouseEvent),
    Down(WMouseEvent),
    Move(WMouseEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WPdfEvent {
    Scale(ScaleEvent),
    Scroll(ScrollEvent),
    Mouse {
        page_index: u16, // PdfPageIndex
        e: MouseEvent,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WPdfLoadEvent {
    DocLoading,
    DocLoaded(PdfDocumentInfo),
    DocStructureLoading,
    DocStructureLoaded,
}
