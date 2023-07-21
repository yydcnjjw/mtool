use base64::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash)]
pub struct PdfFile {
    pub path: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bounds {
    pub bottom: isize,
    pub left: isize,
    pub top: isize,
    pub right: isize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageInfo {
    pub width: i32,
    pub height: i32,
    pub text_segs: Vec<Bounds>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PdfInfo {
    pub pages: Vec<PageInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct PdfRenderArgs {
    pub page_index: u16,
}

impl PdfRenderArgs {
    #[allow(unused)]
    pub fn decode<T: AsRef<[u8]>>(input: T) -> Result<Self, anyhow::Error> {
        Ok(serde_json::from_slice(&BASE64_STANDARD.decode(input)?)?)
    }

    #[allow(unused)]
    pub fn encode(&self) -> Result<String, anyhow::Error> {
        Ok(BASE64_STANDARD.encode(serde_json::to_string(self)?))
    }
}
