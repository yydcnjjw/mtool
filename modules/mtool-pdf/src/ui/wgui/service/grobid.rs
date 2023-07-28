use std::str::FromStr;

use itertools::Itertools;
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct Rect {
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl<'de> Deserialize<'de> for Rect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let buf = String::deserialize(deserializer)?;
        Self::from_str(&buf).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Rect {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.split(",").collect_vec();
        if v.len() != 5 {
            anyhow::bail!("{} format is worry", s);
        }

        Ok(Self {
            page: usize::from_str(v[0])?,
            x: f64::from_str(v[1])?,
            y: f64::from_str(v[2])?,
            w: f64::from_str(v[3])?,
            h: f64::from_str(v[4])?,
        })
    }
}

impl Rect {
    fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Self>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let buf = String::deserialize(deserializer)?;

        buf.split(";")
            .map(|s| Rect::from_str(s))
            .try_collect()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Deserialize)]
enum SentenceElement {
    #[serde(rename = "$text")]
    Content(String),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Sentence {
    #[serde(rename = "@coords", deserialize_with = "Rect::deserialize")]
    pub coords: Vec<Rect>,
    #[serde(rename = "$value")]
    elems: Vec<SentenceElement>,
}

impl Sentence {
    pub fn text(&self) -> String {
        self.elems
            .iter()
            .filter_map(|e| match e {
                SentenceElement::Content(c) => Some(c),
                SentenceElement::Other => None,
            })
            .join("")
    }
}

#[derive(Debug, Deserialize)]
enum ParagraphElement {
    #[serde(rename = "s")]
    Sentence(Sentence),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Paragraph {
    #[serde(rename = "$value")]
    elems: Vec<ParagraphElement>,
}

impl Paragraph {
    pub fn sentence(&self) -> impl Iterator<Item = &Sentence> {
        self.elems.iter().filter_map(|e| match e {
            ParagraphElement::Sentence(s) => Some(s),
            _ => None,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Header {
    #[serde(rename = "@coords", deserialize_with = "Rect::deserialize")]
    pub coords: Vec<Rect>,
    #[serde(rename = "$text")]
    pub text: String,
}

#[derive(Debug, Deserialize)]
enum DivElement {
    #[serde(rename = "head")]
    Header(Header),
    #[serde(rename = "p")]
    Paragraph(Paragraph),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Div {
    #[serde(rename = "$value")]
    elems: Vec<DivElement>,
}

impl Div {
    pub fn header(&self) -> impl Iterator<Item = &Header> {
        self.elems.iter().filter_map(|e| match e {
            DivElement::Header(h) => Some(h),
            _ => None,
        })
    }

    pub fn paragraph(&self) -> impl Iterator<Item = &Paragraph> {
        self.elems.iter().filter_map(|e| match e {
            DivElement::Paragraph(p) => Some(p),
            _ => None,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Body {
    #[serde(rename = "div")]
    pub divs: Vec<Div>,
}

#[derive(Debug, Deserialize)]
pub struct Text {
    pub body: Body,
}

#[derive(Debug, Deserialize)]
pub struct TEI {
    pub text: Text,
}
