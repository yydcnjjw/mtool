use std::ops::RangeFrom;

use nom::{InputIter, InputLength, Slice};

use super::{common::NomResult, dict_meta, key_block, record_block, Dict, Result};

fn parse_inner<I>(in_: I) -> NomResult<I, Dict>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, meta) = dict_meta::parse(in_)?;
    let (in_, key_block) = key_block::parse(in_, &meta)?;
    let (in_, record_block) = record_block::parse(in_, &meta)?;

    Ok((
        in_,
        Dict {
            meta,
            key_block,
            record_block,
        },
    ))
}

pub fn parse<I>(in_: I) -> Result<Dict>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (_, dict) = parse_inner(in_)?;
    Ok(dict)
}
