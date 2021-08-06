use std::ops::RangeFrom;

use nom::{InputIter, InputLength, Slice};

use crate::{
    common::mdict_string,
    dict_meta::{self, DictMeta},
    key_block::{self, KeyBlock},
    record_block::{self, RecordBlock},
    NomResult,
};

#[derive(Debug)]
pub struct Mdx {
    pub meta: DictMeta,
    key_block: KeyBlock,
    record_block: RecordBlock,
}

impl Mdx {
    pub fn search(&self, text: String) -> Vec<(String, String)> {
        self.key_block
            .keymap
            .iter()
            .filter(|item| item.0.contains(&text))
            .map(|item| (item.0.clone(), self.record(&self.meta, *item.1)))
            .collect::<_>()
    }

    fn record(&self, meta: &DictMeta, mut pos: u64) -> String {
        self.record_block
            .blocks
            .iter()
            .find(|item| {
                let len = item.len() as u64;
                if pos >= len {
                    pos -= len;
                    false
                } else {
                    true
                }
            })
            .map(|item| {
                let r: NomResult<&[u8], String> = mdict_string(meta)(&item[pos as usize..]);
                r.unwrap().1
            })
            .unwrap_or_default()
    }
}

pub fn parse<I>(in_: I) -> NomResult<I, Mdx>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, meta) = dict_meta::parse(in_)?;
    let (in_, key_block) = key_block::parse(in_, &meta)?;
    let (in_, record_block) = record_block::parse(in_, &meta)?;

    Ok((
        in_,
        Mdx {
            meta,
            key_block,
            record_block,
        },
    ))
}
