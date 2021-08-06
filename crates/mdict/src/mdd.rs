use std::ops::RangeFrom;

use nom::{InputIter, InputLength, Slice};

use crate::{
    dict_meta::{self, DictMeta},
    key_block::{self, KeyBlock},
    record_block::{self, RecordBlock},
    NomResult,
};

#[derive(Debug)]
pub struct Mdd {
    pub meta: DictMeta,
    key_block: KeyBlock,
    record_block: RecordBlock,
}

pub fn parse<I>(in_: I) -> NomResult<I, Mdd>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, meta) = dict_meta::parse(in_)?;
    let (in_, key_block) = key_block::parse(in_, &meta)?;
    let (in_, record_block) = record_block::parse(in_, &meta)?;

    Ok((
        in_,
        Mdd {
            meta,
            key_block,
            record_block,
        },
    ))
}
