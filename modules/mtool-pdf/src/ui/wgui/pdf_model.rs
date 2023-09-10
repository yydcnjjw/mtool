use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash)]
pub struct PdfFile {
    pub path: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bound {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub r#type: String,
    pub index: usize,
    pub bound: Bound,
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
pub enum WPdfEvent {
    Scale(ScaleEvent),
    Scroll(ScrollEvent),
    HighlightSentence {
        page_index: u16, // PdfPageIndex
        x: i32,
        y: i32,
    },
    PrevSentence,
    NextSentence,
}
