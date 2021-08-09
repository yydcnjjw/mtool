use std::ops::RangeFrom;

use nom::{InputIter, InputLength, Slice};

use crate::{
    dict_meta::{self, DictMeta},
    key_block::{self, KeyBlock},
    record_block::{self, RecordBlock},
    NomResult,
};

#[derive(Debug)]
pub struct Mdict {
    pub meta: DictMeta,
    pub key_block: KeyBlock,
    pub record_block: RecordBlock,
}

pub(crate) fn parse<I>(in_: I) -> NomResult<I, Mdict>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, meta) = dict_meta::parse(in_)?;
    let (in_, key_block) = key_block::parse(in_, &meta)?;
    let (in_, record_block) = record_block::parse(in_, &meta)?;

    Ok((
        in_,
        Mdict {
            meta,
            key_block,
            record_block,
        },
    ))
}
