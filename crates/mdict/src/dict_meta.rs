use std::ops::RangeFrom;

use nom::{
    combinator::map,
    multi::length_count,
    number::streaming::{be_u32, le_u16, le_u32},
    sequence::tuple,
    InputIter, InputLength, Slice,
};

use serde::Deserialize;

use crate::{nom_return, NomResult};

#[derive(Debug, Deserialize, PartialEq)]
pub struct DictMeta {
    #[serde(rename = "GeneratedByEngineVersion")]
    generated_by_engine_version: f64,
    #[serde(rename = "RequiredEngineVersion")]
    required_engine_version: f64,
    #[serde(rename = "Format")]
    format: String,
    #[serde(rename = "KeyCaseSensitive")]
    key_case_sensitive: String,
    #[serde(rename = "StripKey")]
    strip_key: Option<String>,
    #[serde(rename = "Encrypted")]
    encrypted: String,
    #[serde(rename = "RegisterBy")]
    register_by: Option<String>,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Encoding")]
    pub encoding: String,
    #[serde(rename = "CreationDate")]
    creation_date: Option<String>,
    #[serde(rename = "Compact")]
    compact: Option<String>,
    #[serde(rename = "Compat")]
    compat: Option<String>,
    #[serde(rename = "Left2Right")]
    left2right: Option<String>,
    #[serde(rename = "DataSourceFormat")]
    data_source_format: Option<String>,
    #[serde(rename = "StyleSheet")]
    style_sheet: Option<String>,
}

impl DictMeta {
    pub fn is_ver2(&self) -> bool {
        self.required_engine_version >= 2.0
    }
}

pub fn parse<I>(in_: I) -> NomResult<I, DictMeta>
where
    I: Clone + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, (dict_meta, _checksum)) =
        tuple((length_count(map(be_u32, |i| i / 2), le_u16), le_u32))(in_)?;

    nom_return!(in_, DictMeta, {
        quick_xml::de::from_str::<DictMeta>(&String::from_utf16(&dict_meta)?)?
    })
}
