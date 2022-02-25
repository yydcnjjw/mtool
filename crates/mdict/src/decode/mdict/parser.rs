use std::{fs, ops::RangeFrom, path::Path};

use anyhow::Context;
use nom::{InputIter, InputLength, Slice};

use super::{common::NomResult, dict_meta, key_block, record_block, Dict, Result};

pub fn parse<I>(in_: I) -> NomResult<I, Dict>
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

pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<Dict> {
    let buf = fs::read(path).context("Failed to read dict")?;

    let (_, mdict) = parse(buf.as_slice())?;
    Ok(mdict)
}
